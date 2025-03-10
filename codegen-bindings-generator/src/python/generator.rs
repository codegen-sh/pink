use codegen_sdk_common::Language;
use quote::format_ident;
use syn::parse_quote;
pub(crate) fn generate_bindings(language: &Language) -> anyhow::Result<Vec<syn::Stmt>> {
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
    let package_name = format_ident!("codegen_sdk_{}", language.name());
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
            pub fn content(&self, py: Python<'_>) -> PyResult<String> {
                let codebase = self.codebase.get(py);
                let file = self.file(py)?.root(codebase.db());
                Ok(file.source())
            }
            pub fn content_bytes(&self, py: Python<'_>) -> PyResult<pyo3_bytes::PyBytes> {
                let codebase = self.codebase.get(py);
                let file = self.file(py)?.root(codebase.db());
                Ok(pyo3_bytes::PyBytes::new(file.text()))
            }
        }
    });
    let language_name = language.name();
    let register_name = format_ident!("register_{}", language_name);

    output.push(parse_quote! {
        pub fn #register_name(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
            let child_module = PyModule::new(parent_module.py(), #language_name)?;
            pyo3_log::init();
            child_module.add_class::<#struct_name>()?;
            parent_module.add_submodule(&child_module)?;
            Ok(())
        }
    });
    Ok(output)
}
