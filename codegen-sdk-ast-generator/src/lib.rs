#![feature(extend_one)]
use codegen_sdk_common::language::Language;
use quote::quote;

use crate::query::HasQuery;
mod generator;
mod query;
mod visitor;
pub fn generate_ast(language: &Language) -> anyhow::Result<()> {
    let imports = quote! {
        use derive_visitor::{Visitor};
        use codegen_sdk_common::*;
        use std::path::PathBuf;

    };
    let mut ast = generator::generate_ast(language)?;
    let visitor = visitor::generate_visitor(
        &language.definitions().values().into_iter().collect(),
        language,
    );
    ast = imports.to_string() + &ast + &visitor.to_string();
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/{}.rs", out_dir, language.name());
    std::fs::write(out_file, ast)?;
    Ok(())
}
