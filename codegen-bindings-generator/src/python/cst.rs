use codegen_sdk_common::Language;
use proc_macro2::Span;
use quote::format_ident;
use syn::parse_quote;

fn generate_cst_struct(
    language: &Language,
    node: &codegen_sdk_cst_generator::Node,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let struct_name = format_ident!("{}", node.normalize_name());
    let package_name = syn::Ident::new(&language.package_name(), Span::call_site());
    let variant_name = format_ident!("{}", language.struct_name);
    output.push(parse_quote! {
        #[pyclass]
        pub struct #struct_name {
            id: codegen_sdk_common::CSTNodeTreeId,
            codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
        }
    });
    output.push(parse_quote! {
        impl #struct_name {
            pub fn new(id: codegen_sdk_common::CSTNodeTreeId, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> Self {
                Self { id, codebase }
            }
            fn get_node<'db>(&'db self, py: Python<'db>) -> PyResult<&'db #package_name::cst::#struct_name<'db>> {
                let codebase = self.codebase.get(py);
                let file = codebase.get_file_for_id(self.id.file(codebase.db()));
                if let Some(codegen_sdk_analyzer::ParsedFile::#variant_name(py)) = file {
                    let tree = py.tree(codebase.db());
                    let node = tree.get(self.id.id(codebase.db()));
                    if let Some(node) = node {
                        node.as_ref().try_into().map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to convert node to CSTNode {}", e)))
                    } else {
                        Err(pyo3::exceptions::PyValueError::new_err("Node not found"))
                    }
                } else {
                    Err(pyo3::exceptions::PyValueError::new_err("File not found"))
                }
            }
        }
    });
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
