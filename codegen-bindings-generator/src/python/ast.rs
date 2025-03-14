use std::collections::BTreeMap;

use codegen_sdk_ast_generator::Symbol;
use codegen_sdk_common::Language;
use convert_case::{Case, Casing};
use pluralizer::pluralize;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_quote, parse_quote_spanned};

use super::helpers;
fn get_category(group: &str) -> Vec<syn::Stmt> {
    let category = format_ident!("{}", pluralize(group, 2, false));
    parse_quote! {
        let file = self.file(py)?;
        let db = self.codebase.get(py).db();
        let category = file.#category(db);
    }
}
fn filter_symbols(
    nodes: &Vec<&codegen_sdk_ast_generator::Symbol>,
    group: &str,
) -> Vec<codegen_sdk_ast_generator::Symbol> {
    nodes
        .iter()
        .filter(|node| node.category == pluralize(group, 2, false))
        .cloned()
        .cloned()
        .collect()
}
fn get_symbol_method_name(group: &str) -> syn::Ident {
    let symbol_name = codegen_sdk_ast_generator::get_symbol_name(group);
    syn::Ident::new(
        &format!("get_{}", symbol_name.to_string().to_case(Case::Snake)),
        Span::call_site(),
    )
}
// Generates the get_symbol method
fn get_symbols_method(group: &str) -> Vec<syn::Stmt> {
    let mut output: Vec<syn::Stmt> = Vec::new();
    let symbol_name = codegen_sdk_ast_generator::get_symbol_name(group);
    let category = get_category(group);
    let symbols_method = codegen_sdk_ast_generator::get_symbols_method(&symbol_name);
    let get_symbols_method = get_symbol_method_name(group);
    output.extend::<Vec<syn::Stmt>>(parse_quote! {
        #[pyo3(signature = (name,optional=false))]
        pub fn #get_symbols_method(&self, py: Python<'_>, name: String, optional: bool) -> PyResult<Option<#symbol_name>> {
            #(#category)*
            let subcategory = category.#symbols_method(db);
            let res = subcategory.get(&name);
            if let Some(nodes) = res {
                if nodes.len() == 1 {
                    Ok(Some(#symbol_name::new(py.clone(), nodes[0].fully_qualified_name(db), 0, &nodes[0], self.codebase.clone())))
                } else {
                    Err(pyo3::exceptions::PyValueError::new_err(format!("Ambiguous symbol {} found {} possible matches", name, nodes.len())))
                }
            } else {
                if optional {
                    Ok(None)
                } else {
                    Err(pyo3::exceptions::PyValueError::new_err(format!("No symbol {} found", name)))
                }
            }
    }
    });
    output
}
fn symbols_method(group: &str) -> Vec<syn::Stmt> {
    let mut output: Vec<syn::Stmt> = Vec::new();
    let symbol_name = codegen_sdk_ast_generator::get_symbol_name(group);
    let category = get_category(group);
    let symbols_method = codegen_sdk_ast_generator::get_symbols_method(&symbol_name);
    output.extend::<Vec<syn::Stmt>>(parse_quote! {
        #[getter]
        pub fn #symbols_method(&self, py: Python<'_>) -> PyResult<Vec<#symbol_name>> {
            #(#category)*
            let subcategory = category.#symbols_method(db);
            let nodes = subcategory.values().map(|values| values.into_iter().enumerate().map(|(idx, node)| #symbol_name::new(py.clone(), node.fully_qualified_name(db), idx, node, self.codebase.clone()))).flatten().collect();
            Ok(nodes)
    }
    });
    output
}
fn get_symbol_references(
    symbol: &codegen_sdk_ast_generator::Symbol,
    language: &Language,
) -> Vec<syn::Stmt> {
    if symbol.category == "definitions" {
        let span = Span::call_site();
        let package_name = syn::Ident::new(&language.package_name(), span);
        let symbol_enum = quote! { codegen_sdk_analyzer::#package_name::ast::Symbol };
        let symbol_name = format_ident!("{}", symbol.name);
        return parse_quote! {
            #[getter]
            pub fn references(&self, py: Python<'_>) -> PyResult<Vec<Reference>> {
                let val = self.get(py)?;
                let wrapped = #symbol_enum::#symbol_name(*val);
                let db = self.codebase.get(py).db();
                let references = wrapped.references(db);
                Ok(references.into_iter().map(|reference| Reference::new(py.clone(), reference.fully_qualified_name(db), 0, &reference, self.codebase.clone())).collect())
            }
        };
    }
    parse_quote! {}
}
fn generate_symbol_struct(
    language: &Language,
    symbol: &codegen_sdk_ast_generator::Symbol,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let span = Span::call_site();
    let mut output = Vec::new();
    let struct_name = format_ident!("{}", symbol.name);
    let package_name = syn::Ident::new(&language.package_name(), span);
    let module_name = format!("codegen_sdk_pink.{}", language.name());
    output.push(parse_quote_spanned! {
        span =>
        #[pyclass(module=#module_name)]
        pub struct #struct_name {
            id: codegen_sdk_resolution::FullyQualifiedName,
            idx: usize,
            codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
        }
    });
    let file_getter = helpers::get_file(language, quote! { self.id }, quote! { self.codebase });
    let category = syn::Ident::new(&symbol.category, span);
    let subcategory = syn::Ident::new(&symbol.subcategory, span);
    output.push(parse_quote_spanned! {
        span =>
        impl #struct_name {
            pub fn new(id: codegen_sdk_resolution::FullyQualifiedName, idx: usize, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> Self {
                Self { id, idx, codebase }
            }
            fn get<'db>(&'db self, py: Python<'db>) -> PyResult<&'db codegen_sdk_analyzer::#package_name::ast::#struct_name<'db>> {
                #(#file_getter)*
                let name = self.id.name(codebase.db());
                let node = file.#category(codebase.db()).#subcategory(codebase.db()).get(name).unwrap();
                node.get(self.idx).ok_or(pyo3::exceptions::PyValueError::new_err("Index out of bounds"))
            }
        }
    });
    let fields: Vec<syn::Stmt> = symbol
        .fields
        .iter()
        .map(|field| -> Vec<syn::Stmt> {
            let name = syn::Ident::new(&field.name, span);
            let underscore_name = syn::Ident::new(&format!("_{}", field.name), span);
            let type_name = syn::Ident::new(&field.kind, span);
            parse_quote_spanned! {
                span =>
                #[getter]
                pub fn #name(&self, py: Python<'_>) -> PyResult<cst::#type_name> {
                    let node = self.get(py)?;
                    let db = self.codebase.get(py).db();
                    Ok(cst::#type_name::new(py.clone(), node.#underscore_name(db).clone(), self.codebase.clone())?)
                }
            }
        })
        .flatten()
        .collect();
    let references = get_symbol_references(symbol, language);
    let ts_node_name = syn::Ident::new(&symbol.type_name, span);
    output.push(parse_quote_spanned! {
        span =>
        #[pymethods]
        impl #struct_name {
            pub fn ts_node(&self, py: Python<'_>) -> PyResult<cst::#ts_node_name> {
                let node = self.get(py)?;
                let db = self.codebase.get(py).db();
                Ok(cst::#ts_node_name::new(py, node.node_id(db), self.codebase.clone())?)
            }
            fn source(&self, py: Python<'_>) -> PyResult<std::string::String> {
                let db = self.codebase.get(py).db();
                let node = self.get(py)?.node(db);
                Ok(node.source())
            }
            fn __str__(&self, py: Python<'_>) -> PyResult<std::string::String> {
                Ok(self.source(py)?)
            }
            fn __repr__(&self, py: Python<'_>) -> PyResult<std::string::String> {
                let node = self.get(py)?;
                let codebase = self.codebase.get(py);
                codebase.attach(|_db| {
                    Ok(format!("{node:#?}"))
                })
            }
            #(#fields)*
            #(#references)*
        }
    });
    Ok(output)
}
// Generate an enum for all possible symbols. (IE Symbol => Class, Function, etc)
fn generate_symbol_enum(
    language: &Language,
    symbols: Vec<&Symbol>,
    group: &str,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let symbols: Vec<_> = filter_symbols(&symbols, group)
        .iter()
        .map(|symbol| format_ident!("{}", symbol.name))
        .collect();
    if symbols.len() == 0 {
        return Ok(Vec::new());
    }
    let symbol_name = codegen_sdk_ast_generator::get_symbol_name(group);
    let span = Span::call_site();
    let mut output = Vec::new();
    let enum_name = format_ident!("{}", symbol_name);
    let package_name = syn::Ident::new(&language.package_name(), span);
    output.push(parse_quote_spanned! {
        span =>
        #[derive(IntoPyObject)]
        pub enum #enum_name {
            #(#symbols(#symbols),)*
        }
    });
    let original_name = quote! { codegen_sdk_analyzer::#package_name::ast::#symbol_name };
    let matchers: Vec<syn::Arm> = symbols
        .iter()
        .map(|symbol| {
            let symbol_name = format_ident!("{}", symbol);
            parse_quote_spanned! {
                span =>
                #original_name::#symbol_name(_) => Self::#symbol_name(#symbol_name::new(id, idx, codebase_arc)),
            }
        })
        .collect();
    output.push(parse_quote_spanned! {
        span =>
        impl #enum_name {
            pub fn new(py: Python<'_>, id: codegen_sdk_resolution::FullyQualifiedName, idx: usize, node: &#original_name<'_>, codebase_arc: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> Self {
                match node {
                    #(#matchers)*
                }
            }
        }
    });
    Ok(output)
}
pub fn generate_ast(
    language: &Language,
    symbols: &BTreeMap<String, Symbol>,
) -> anyhow::Result<(Vec<syn::Stmt>, Vec<syn::Ident>)> {
    let mut output = Vec::new();
    let mut symbol_idents = Vec::new();
    for group in codegen_sdk_ast_generator::GROUPS {
        let symbol_enum = generate_symbol_enum(language, symbols.values().collect(), group)?;
        output.extend(symbol_enum);
    }
    for (_, symbol) in symbols {
        let symbol_struct = generate_symbol_struct(language, &symbol)?;
        output.extend(symbol_struct);
        symbol_idents.push(format_ident!("{}", symbol.name));
    }
    Ok((output, symbol_idents))
}
pub fn get_symbol_methods(symbols: &Vec<&Symbol>) -> Vec<syn::Stmt> {
    let mut symbols_methods = Vec::new();
    for group in codegen_sdk_ast_generator::GROUPS {
        if filter_symbols(&symbols, group).len() > 0 {
            symbols_methods.extend(symbols_method(group));
            symbols_methods.extend(get_symbols_method(group));
        }
    }
    symbols_methods
}
