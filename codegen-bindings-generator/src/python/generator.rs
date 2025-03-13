use codegen_sdk_ast_generator::{HasQuery, Symbol};
use codegen_sdk_common::Language;
use codegen_sdk_cst::CSTDatabase;
use convert_case::{Case, Casing};
use pluralizer::pluralize;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_quote, parse_quote_spanned};

use super::{cst::generate_cst, helpers};
fn get_category(group: &str) -> Vec<syn::Stmt> {
    let category = format_ident!("{}", pluralize(group, 2, false));
    parse_quote! {
        let file = self.file(py)?;
        let db = self.codebase.get(py).db();
        let category = file.#category(db);
    }
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
fn generate_file_struct(
    language: &Language,
    symbols: Vec<&Symbol>,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let struct_name = format_ident!("{}File", language.struct_name);
    let module_name = format!("codegen_sdk_pink.{}", language.name());
    output.push(parse_quote! {
        // #[gen_stub_pyclass]
        #[pyclass(module=#module_name)]
        pub struct #struct_name {
            path: PathBuf,
            codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
        }
    });
    let span = Span::call_site();
    let package_name = syn::Ident::new(&language.package_name(), span);
    let variant_name = format_ident!("{}", language.struct_name);
    output.push(parse_quote! {
        impl #struct_name {
            pub fn new(path: PathBuf, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> Self {
                Self { path, codebase }
            }
            fn file<'db>(&'db self, py: Python<'db>) -> PyResult<&'db codegen_sdk_analyzer::#package_name::ast::#struct_name<'db>>{
                let codebase = self.codebase.get(py);
                if let codegen_sdk_analyzer::ParsedFile::#variant_name(file) = codebase.get_file(&self.path).unwrap() {
                    Ok(file)
                } else {
                    Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "File not found at {}",
                        self.path.display()
                    )))
                }
            }
        }
    });
    let methods = symbols
        .iter()
        .filter(|symbol| symbol.category != symbol.subcategory)
        .map(|symbol| vec![symbol.py_file_getter(), symbol.py_file_get()])
        .flatten();
    let mut symbols_methods = Vec::new();
    for group in codegen_sdk_ast_generator::GROUPS {
        symbols_methods.extend(symbols_method(group));
        symbols_methods.extend(get_symbols_method(group));
    }
    output.push(parse_quote! {
        #[pymethods]
        impl #struct_name {
            #[getter]
            pub fn path(&self) -> &PathBuf {
                &self.path
            }
            #[getter]
            pub fn content(&self, py: Python<'_>) -> PyResult<std::string::String> {
                let codebase = self.codebase.get(py);
                let file = self.file(py)?.root(codebase.db());
                Ok(file.source())
            }
            #[getter]
            pub fn content_bytes(&self, py: Python<'_>) -> PyResult<pyo3_bytes::PyBytes> {
                let codebase = self.codebase.get(py);
                let file = self.file(py)?.root(codebase.db());
                Ok(pyo3_bytes::PyBytes::new(file.text()))
            }
            fn __str__(&self, py: Python<'_>) -> PyResult<String> {
                Ok(self.content(py)?.to_string())
            }
            #(#methods)*
            #(#symbols_methods)*
        }
    });
    Ok(output)
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
    let symbols: Vec<_> = symbols
        .iter()
        .filter(|symbol| symbol.category == pluralize(group, 2, false))
        .map(|symbol| format_ident!("{}", symbol.name))
        .collect();
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
fn generate_module(
    language: &Language,
    symbols: Vec<syn::Ident>,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let language_name = language.name();
    let register_name = format_ident!("register_{}", language_name);
    let struct_name = format_ident!("{}", language.file_struct_name());
    let module_name = format!("codegen_sdk_pink.{}", language_name);
    output.push(parse_quote! {
        pub fn #register_name(py: Python<'_>, parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
            let child_module = PyModule::new(parent_module.py(), #language_name)?;
            child_module.add_class::<#struct_name>()?;
            #(child_module.add_class::<#symbols>()?;)*
            parent_module.add_submodule(&child_module)?;
            cst::register_cst(&child_module)?;
            py.import("sys")?
            .getattr("modules")?
            .set_item(#module_name, child_module)?;
            Ok(())
        }
    });
    Ok(output)
}
pub(crate) fn generate_bindings(language: &Language) -> anyhow::Result<Vec<syn::Stmt>> {
    let config = codegen_sdk_cst_generator::Config::default();
    let state = codegen_sdk_cst_generator::State::new(language, config);
    let db = CSTDatabase::default();
    let symbols = language.symbols(&db);
    let file_struct = generate_file_struct(language, symbols.values().collect())?;
    let mut output = Vec::new();
    output.extend(file_struct);

    let cst = generate_cst(language, &state)?;
    output.extend(cst);
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
    let module = generate_module(language, symbol_idents)?;
    output.extend(module);
    Ok(output)
}
#[cfg(test)]
mod tests {
    use codegen_sdk_common::{
        Language,
        generator::format_code,
        language::{python::Python, rust::Rust, typescript::Typescript},
    };
    use rstest::rstest;

    use super::*;
    #[test_log::test(rstest)]
    #[case::python(&Python)]
    #[case::typescript(&Typescript)]
    #[case::rust(&Rust)]
    fn test_generate_bindings(#[case] language: &Language) {
        let bindings = generate_bindings(&language).unwrap();
        let output = parse_quote! { #(#bindings)* };
        insta::assert_snapshot!(language.name(), format_code(&output).unwrap());
    }
}
