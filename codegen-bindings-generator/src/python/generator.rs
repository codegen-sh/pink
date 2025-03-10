use codegen_sdk_ast_generator::HasQuery;
use codegen_sdk_common::Language;
use codegen_sdk_cst::CSTDatabase;
use proc_macro2::Span;
use quote::format_ident;
use syn::parse_quote;

use super::cst::generate_cst;
fn generate_file_struct(language: &Language) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let struct_name = format_ident!("{}File", language.struct_name);

    output.push(parse_quote! {
        // #[gen_stub_pyclass]
        #[pyclass]
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
            fn file<'db>(&'db self, py: Python<'db>) -> PyResult<&'db #package_name::ast::#struct_name<'db>>{
                let codebase = self.codebase.get(py);
                if let codegen_sdk_analyzer::ParsedFile::#variant_name(file) = codebase.get_file(self.path.clone()).unwrap() {
                    Ok(file)
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
        }
    });
    Ok(output)
}
fn generate_symbol_struct(
    language: &Language,
    symbol: &codegen_sdk_ast_generator::Symbol,
) -> anyhow::Result<Vec<syn::Stmt>> {
    let mut output = Vec::new();
    let struct_name = format_ident!("{}", symbol.name);
    output.push(parse_quote! {
        #[pyclass]
        pub struct #struct_name {
            id: codegen_sdk_resolution::FullyQualifiedName,
            codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
        }
    });
    output.push(parse_quote! {
        impl #struct_name {
            pub fn new(id: codegen_sdk_resolution::FullyQualifiedName, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> Self {
                Self { id, codebase }
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
    output.push(parse_quote! {
        pub fn #register_name(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
            let child_module = PyModule::new(parent_module.py(), #language_name)?;
            child_module.add_class::<#struct_name>()?;
            #(child_module.add_class::<#symbols>()?;)*
            parent_module.add_submodule(&child_module)?;
            cst::register_cst(&child_module)?;
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
    let file_struct = generate_file_struct(language)?;
    let mut output = Vec::new();
    output.extend(file_struct);

    let cst = generate_cst(language, &state)?;
    output.extend(cst);
    let mut symbol_idents = Vec::new();
    for (_, symbol) in symbols {
        let symbol_struct = generate_symbol_struct(language, &symbol)?;
        output.extend(symbol_struct);
        symbol_idents.push(format_ident!("{}", symbol.name));
    }
    let module = generate_module(language, symbol_idents)?;
    output.extend(module);
    Ok(output)
}
