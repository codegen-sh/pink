#![feature(extend_one)]

use codegen_sdk_common::{generator::format_code, language::Language};
use quote::quote;

mod generator;
mod query;
mod visitor;
pub fn generate_ast(language: &Language) -> anyhow::Result<()> {
    let imports = quote! {
        use derive_visitor::{Visitor, Drive};
        use codegen_sdk_common::*;
        use std::path::PathBuf;
        use codegen_sdk_cst::CSTLanguage;
    };
    let mut ast = generator::generate_ast(language)?;
    let definitions = visitor::generate_visitor(language, "definition");
    let references = visitor::generate_visitor(language, "reference");
    ast = imports.to_string() + &ast + &definitions.to_string() + &references.to_string();
    ast = format_code(&ast)
        .unwrap_or_else(|_| panic!("Failed to format ast for {}", language.name()));
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/{}.rs", out_dir, language.name());
    std::fs::write(out_file, ast)?;
    Ok(())
}
