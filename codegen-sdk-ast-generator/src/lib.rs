#![feature(extend_one)]

use codegen_sdk_common::{generator::format_code, language::Language};
use codegen_sdk_cst::CSTDatabase;
use quote::quote;
mod generator;
mod query;
mod visitor;
use syn::parse_quote;
pub fn generate_ast(language: &Language) -> anyhow::Result<()> {
    let db = CSTDatabase::default();
    let mut imports = quote! {
        use derive_generic_visitor::{Visitor, Drive, Visit};
        use codegen_sdk_common::*;
        use std::path::PathBuf;
        use codegen_sdk_cst::CSTLanguage;
    };
    let ast = generator::generate_ast(language)?;
    let definition_visitor = visitor::generate_visitor(&db, language, "definition");
    let reference_visitor = visitor::generate_visitor(&db, language, "reference");
    let ast: syn::File = parse_quote! {
        #imports
        #ast
        #definition_visitor
        #reference_visitor
    };
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/{}.rs", out_dir, language.name());
    std::fs::write(&out_file, ast.to_string())?;
    let ast = format_code(&ast).unwrap_or_else(|_| {
        panic!(
            "Failed to format ast for {} at {}",
            language.name(),
            out_file
        )
    });
    std::fs::write(out_file, ast)?;
    Ok(())
}
