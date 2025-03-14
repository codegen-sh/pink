// Generate the AST and type resolution for a language
#![feature(extend_one)]

use codegen_sdk_common::{generator::format_code, language::Language};
use codegen_sdk_cst::CSTDatabase;
use quote::{ToTokens, quote};
mod generator;
mod query;
mod resolution;
pub use query::{GROUPS, HasQuery, field::Field, symbol::Symbol};
mod visitor;
use syn::parse_quote;
pub use visitor::{get_symbol_name, get_symbols_method};
pub fn generate_ast(language: &Language) -> anyhow::Result<()> {
    let db = CSTDatabase::default();
    let imports = quote! {
    use codegen_sdk_common::*;
    use std::path::PathBuf;
    use codegen_sdk_cst::CSTLanguage;
    use std::collections::BTreeMap;
    use codegen_sdk_resolution::HasFile;
    use codegen_sdk_resolution::Parse;
    use codegen_sdk_resolution::{ResolveType, Scope};
    use codegen_sdk_ast::{Definitions as _, References as _};
    use codegen_sdk_resolution::ResolutionStack;
    use codegen_sdk_resolution::HasId;
    use codegen_sdk_resolution::{Dependencies as _};

    };
    let ast = generator::generate_ast(language)?;
    let definition_visitor = visitor::generate_visitor(&db, language, "definition");
    let reference_visitor = visitor::generate_visitor(&db, language, "reference");
    let resolution = resolution::generate_resolution(language);
    let ast: syn::File = parse_quote! {
        #imports
        #ast
        #definition_visitor
        #reference_visitor
        #(#resolution)*
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
