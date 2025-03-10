#![feature(extend_one)]

use codegen_sdk_common::{generator::format_code, language::Language};
use codegen_sdk_cst::CSTDatabase;
use quote::{ToTokens, quote};
mod generator;
mod query;
pub use query::{HasQuery, field::Field, symbol::Symbol};
mod visitor;
use syn::parse_quote;
pub fn generate_ast(language: &Language) -> anyhow::Result<()> {
    let db = CSTDatabase::default();
    let imports = quote! {
    use codegen_sdk_common::*;
    use std::path::PathBuf;
    use codegen_sdk_cst::CSTLanguage;
    use std::collections::BTreeMap;
    use codegen_sdk_resolution::HasFile;
    use codegen_sdk_resolution::Parse;
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
    let out_file = format!("{}/{}-ast.rs", out_dir, language.name());
    std::fs::write(&out_file, ast.to_token_stream().to_string())?;
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
