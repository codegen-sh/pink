use codegen_sdk_ast_generator::{HasQuery, Symbol};
use codegen_sdk_common::Language;
use codegen_sdk_cst::CSTDatabase;
use proc_macro2::Span;
use quote::format_ident;
use syn::parse_quote;

use super::{
    ast::{generate_ast, get_symbol_methods},
    cst::generate_cst,
};
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
    let symbols_methods = get_symbol_methods(&symbols);
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
    let (ast, symbol_idents) = generate_ast(language, &symbols)?;
    output.extend(ast);
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
