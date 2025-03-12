#![feature(extend_one)]

use codegen_sdk_common::{generator::format_code, language::Language};
pub use python::generate_python_bindings_common;
use quote::ToTokens;
use syn::parse_quote;
mod python;
pub(crate) fn get_imports() -> syn::File {
    parse_quote! {
        use pyo3::prelude::*;
        use std::path::PathBuf;
        use std::sync::Arc;
        use pyo3::sync::GILProtected;
        use codegen_sdk_resolution::CodebaseContext;
        use codegen_sdk_common::traits::CSTNode;
        use codegen_sdk_ast::References;
        use codegen_sdk_ast::Definitions;
        use codegen_sdk_resolution::HasId;
    }
}

pub fn generate_python_bindings(language: &Language) -> anyhow::Result<()> {
    let imports = get_imports();
    let bindings = python::generator::generate_bindings(language)?;
    let ast: syn::File = parse_quote! {
        #imports
        #(#bindings)*
    };
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/{}-bindings.rs", out_dir, language.name());
    std::fs::write(&out_file, ast.to_token_stream().to_string())?;
    let ast = format_code(&ast).unwrap_or_else(|_| {
        panic!(
            "Failed to format bindings for {} at {}",
            language.name(),
            out_file
        )
    });
    std::fs::write(out_file, ast)?;
    Ok(())
}
