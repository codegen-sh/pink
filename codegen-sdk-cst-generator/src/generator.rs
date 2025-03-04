#[double]
use codegen_sdk_common::language::Language;
pub use field::Field;
use mockall_double::double;
pub use node::Node;
pub use state::State;
mod constants;
mod field;
mod node;
mod state;
mod utils;
use std::io::Write;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse_quote;

use crate::Config;
fn get_imports(config: &Config) -> TokenStream {
    let mut imports = quote! {

    use std::sync::Arc;
    use tree_sitter;
    use codegen_sdk_common::*;
    use subenum::subenum;
    use std::backtrace::Backtrace;
        use bytes::Bytes;
        use derive_generic_visitor::Drive;
        use ambassador::Delegate;
        use ambassador::delegate_to_methods;
        use codegen_sdk_cst::CSTLanguage;
    };
    if config.serialize {
        imports.extend_one(quote! {
            use rkyv::{Archive, Deserialize, Serialize};
        });
    }
    imports
}
fn get_parser(language: &Language) -> TokenStream {
    let program_id = format_ident!("{}", language.root_node());
    let language_name = format_ident!("{}", language.name());
    let language_struct_name = format_ident!("{}", language.struct_name());
    let root_node = format_ident!("{}", language.root_node());
    quote! {
        #[salsa::tracked]
        pub struct Parsed<'db> {
            #[tracked]
            #[return_ref]
            pub program: Option<#program_id<'db>>,
        }
        pub fn parse_program_raw(db: &dyn salsa::Database, input: codegen_sdk_cst::Input) -> Option<#program_id<'_>> {
            let buffer = Bytes::from(input.content(db).as_bytes().to_vec());
            let tree = codegen_sdk_common::language::#language_name::#language_struct_name.parse_tree_sitter(&input.content(db));
            match tree {
                Ok(tree) => {
                    if tree.root_node().has_error() {
                        ParseError::SyntaxError.report(db);
                        None
                    } else {
                        let buffer = Arc::new(buffer);
                        #program_id::from_node(db, tree.root_node(), &buffer)
                        .map_or_else(|e| {
                            e.report(db);
                            None
                        }, |program| {
                            Some(program)
                        })
                    }
                }
                Err(e) => {
                    e.report(db);
                    None
                }
            }
        }
        #[salsa::tracked]
        pub fn parse_program(db: &dyn salsa::Database, input: codegen_sdk_cst::Input) -> Parsed<'_> {
            Parsed::new(db, parse_program_raw(db, input))
        }
        pub struct #language_struct_name;
        impl CSTLanguage for #language_struct_name {
            type Program<'db> = #root_node<'db>;
            fn language() -> &'static codegen_sdk_common::language::Language {
                &codegen_sdk_common::language::#language_name::#language_struct_name
            }
            fn parse<'db>(db: &'db dyn salsa::Database, content: std::string::String) -> &'db Option<Self::Program<'db>> {
                let input = codegen_sdk_cst::Input::new(db, content);
                return parse_program(db, input).program(db);
            }
        }
    }
}
pub fn generate_cst(language: &Language, config: Config) -> anyhow::Result<String> {
    let imports: TokenStream = get_imports(&config);
    let state = State::new(language, config);
    let enums = state.get_enum(false);
    let enums_ref = state.get_enum(true);
    let structs = state.get_structs();
    let parser = get_parser(language);
    let result: syn::File = parse_quote! {
        #imports
        #enums
        #enums_ref
        #structs
        #parser
    };
    let formatted = codegen_sdk_common::generator::format_code(&result);
    match formatted {
        Ok(formatted) => return Ok(formatted),
        Err(e) => {
            let mut out_file = tempfile::NamedTempFile::with_suffix(".rs")?;
            log::error!(
                "Failed to format CST, writing to temp file at {}",
                out_file.path().display()
            );
            out_file.write_all(result.into_token_stream().to_string().as_bytes())?;
            out_file.keep()?;
            return Err(e);
        }
    }
}
