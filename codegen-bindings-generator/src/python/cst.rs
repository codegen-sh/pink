use codegen_sdk_common::{Language, naming::normalize_type_name};
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse_quote;

use super::helpers;
fn generate_cst_struct(
    language: &Language,
    node: &codegen_sdk_cst_generator::Node,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let struct_name = format_ident!("{}", node.normalize_name());
    let package_name = syn::Ident::new(&language.package_name(), Span::call_site());
    let module_name = format!("codegen_sdk_pink::{}.cst", language.name());
    output.push(parse_quote! {
        #[pyclass(module=#module_name)]
        pub struct #struct_name {
            id: codegen_sdk_common::CSTNodeTreeId,
            codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
        }
    });
    let file_getter = helpers::get_file(language, quote! { self.id }, quote! { self.codebase });
    let file_type = format_ident!("{}", language.file_struct_name());
    output.push(parse_quote! {
        impl #struct_name {
            pub fn new(_py: Python<'_> ,id: codegen_sdk_common::CSTNodeTreeId, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> PyResult<Self> {
                Ok(Self { id, codebase })
            }
            fn get_file<'db>(&'db self, py: Python<'db>) -> PyResult<&'db codegen_sdk_analyzer::#package_name::ast::#file_type<'db>> {
                #(#file_getter)*
                Ok(file)
            }
            fn get_node<'db>(&'db self, py: Python<'db>) -> PyResult<&'db codegen_sdk_analyzer::#package_name::cst::#struct_name<'db>> {
                let file = self.get_file(py)?;
                let tree = file.tree(self.codebase.get(py).db());
                let node = tree.get(self.id.id(self.codebase.get(py).db()));
                if let Some(node) = node {
                    node.as_ref().try_into().map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to convert node to CSTNode {}", e)))
                } else {
                    Err(pyo3::exceptions::PyValueError::new_err("Node not found"))
                }
            }
        }
    });
    let children_method = if node.has_children() {
        let children_type = format_ident!("{}", node.children_struct_name());
        quote! {
            #[getter]
            pub fn children(&self, py: Python<'_>) -> PyResult<Vec<#children_type>> {
                let file = self.get_file(py)?;
                let db = self.codebase.get(py).db();
                let tree = file.tree(db);
                let children = tree.children(self.id.id(db));
                Ok(children.map(|(child, child_id)| {
                    let id = codegen_sdk_common::CSTNodeTreeId::from_node_id(db, &child.id(),child_id);
                    #children_type::new(py.clone(),id, self.codebase.clone()).unwrap()
                }).collect())
            }
        }
    } else {
        quote! {}
    };
    output.push(parse_quote! {
        #[pymethods]
        impl #struct_name {
            #[getter]
            pub fn source(&self, py: Python<'_>) -> PyResult<std::string::String> {
                let node = self.get_node(py)?;
                Ok(node.source())
            }
            #[getter]
            pub fn _source(&self, py: Python<'_>) -> PyResult<std::string::String> {
                self.source(py)
            }
            #[getter]
            pub fn text(&self, py: Python<'_>) -> PyResult<pyo3_bytes::PyBytes> {
                let node = self.get_node(py)?;
                Ok(pyo3_bytes::PyBytes::new(node.text()))
            }
            #[getter]
            pub fn start_byte(&self, py: Python<'_>) -> PyResult<usize> {
                let node = self.get_node(py)?;
                Ok(node.start_byte())
            }
            #[getter]
            pub fn end_byte(&self, py: Python<'_>) -> PyResult<usize> {
                let node = self.get_node(py)?;
                Ok(node.end_byte())
            }
            #[getter]
            pub fn start_position<'a>(&'a self, py: Python<'a>) -> PyResult<Bound<'a, pyo3::types::PyTuple>> {
                let node = self.get_node(py)?;
                let position = node.start_position();
                let row = position.row(self.codebase.get(py).db());
                let column = position.column(self.codebase.get(py).db());
                pyo3::types::PyTuple::new(py, vec![row, column])
            }
            #[getter]
            pub fn end_position<'a>(&'a self, py: Python<'a>) -> PyResult<Bound<'a, pyo3::types::PyTuple>> {
                let node = self.get_node(py)?;
                let position = node.end_position();
                let row = position.row(self.codebase.get(py).db());
                let column = position.column(self.codebase.get(py).db());
                pyo3::types::PyTuple::new(py ,vec![row, column])
            }
            #children_method
            fn __str__(&self, py: Python<'_>) -> PyResult<std::string::String> {
                Ok(self.source(py)?)
            }
        }
    });
    Ok(output)
}
fn generate_cst_subenum(
    language: &Language,
    state: &codegen_sdk_cst_generator::State,
    name: &str,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let struct_name = format_ident!("{}", normalize_type_name(name, true));
    let subenum_names = state
        .get_subenum_variants(&name, false)
        .iter()
        .map(|name| {
            let name = format_ident!("{}", name.normalize_name());
            parse_quote! {
                #name(#name)
            }
        })
        .collect::<Vec<syn::Variant>>();
    output.push(parse_quote! {
        #[derive(IntoPyObject)]
        pub enum #struct_name {
            #(#subenum_names,)*
        }
    });
    let package_name = syn::Ident::new(&language.package_name(), Span::call_site());
    let ref_name = syn::Ident::new(
        &format!("{}Ref", struct_name.to_string()),
        Span::call_site(),
    );
    let matchers: Vec<syn::Arm> = state
        .get_subenum_variants(&name, false)
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.normalize_name());
            parse_quote! {
                codegen_sdk_analyzer::#package_name::cst::#ref_name::#name(_) => Ok(Self::#name(#name::new(py ,id, codebase_arc.clone())?)),
            }
        })
        .collect();
    let get_file = helpers::get_file(language, quote! { id }, quote! { codebase_arc });
    output.push(parse_quote! {
        impl #struct_name {
            pub fn new(py: Python<'_>, id: codegen_sdk_common::CSTNodeTreeId, codebase_arc: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> PyResult<Self> {
                #(#get_file)*
                let node = file.tree(codebase.db()).get(id.id(codebase.db()));
                if let Some(node) = node {
                    match node.as_ref().try_into().unwrap() {
                        #(#matchers)*
                    }
                } else {
                    Err(pyo3::exceptions::PyValueError::new_err("Node not found"))
                }
            }
        }
    });
    Ok(output)
}

fn generate_module(state: &codegen_sdk_cst_generator::State) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let node_names = state.get_node_struct_names();
    output.push(parse_quote! {
        pub fn register_cst(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
            let child_module = PyModule::new(parent_module.py(), "cst")?;
            #(child_module.add_class::<#node_names>()?;)*
            parent_module.add_submodule(&child_module)?;
            Ok(())
        }
    });
    Ok(output)
}

pub fn generate_cst(
    language: &Language,
    state: &codegen_sdk_cst_generator::State,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    for node in state.nodes() {
        let cst_struct = generate_cst_struct(language, node)?;
        output.extend(cst_struct);
    }
    for subenum in &state.subenums {
        let cst_subenum = generate_cst_subenum(language, state, subenum)?;
        output.extend(cst_subenum);
    }
    output.extend(generate_module(state)?);
    Ok(parse_quote! {
        mod cst {
            use pyo3::prelude::*;
            use std::sync::Arc;
            use pyo3::sync::GILProtected;
            use codegen_sdk_resolution::CodebaseContext;
            use codegen_sdk_common::traits::CSTNode;
            #(#output)*
        }
    })
}
