use codegen_sdk_common::Language;
use proc_macro2::{Span, TokenStream};
use syn::parse_quote_spanned;
pub fn get_file(language: &Language, id: TokenStream, codebase: TokenStream) -> Vec<syn::Stmt> {
    let span = Span::call_site();
    let variant_name = syn::Ident::new(&language.struct_name, span);
    parse_quote_spanned! {
    span =>
    let codebase = #codebase.get(py);
    let path = #id.file(codebase.db());
    let file = codebase.get_file_for_id(path);
    let file = match file {
        Some(codegen_sdk_analyzer::ParsedFile::#variant_name(py)) => py,
        _ => return Err(pyo3::exceptions::PyValueError::new_err(format!("File not found for path: {}", path.path(codebase.db()).display()))),
    };
    }
}
