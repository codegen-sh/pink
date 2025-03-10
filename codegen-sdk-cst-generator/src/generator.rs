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
        use ambassador::Delegate;
        use derive_more::Debug;
        use ambassador::delegate_to_methods;
        use codegen_sdk_cst::CSTLanguage;
        use crate::cst::tree::ParseContext;
        use std::path::PathBuf;
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
        impl<'db> TreeNode for NodeTypes<'db> {}
        #[salsa::tracked]
        pub struct Parsed<'db> {
            #[id]
            id: FileNodeId,
            #[tracked]
            #[return_ref]
            #[no_clone]
            #[no_eq]
            pub tree: Arc<Tree<NodeTypes<'db>>>,
            pub program: indextree::NodeId,
        }
        pub fn parse_program_raw<'db>(db: &'db dyn salsa::Database, input: codegen_sdk_cst::File) -> Option<Parsed<'db>> {
            let buffer = Bytes::from(input.content(db).as_bytes().to_vec());
            let tree = codegen_sdk_common::language::#language_name::#language_struct_name.parse_tree_sitter(&input.content(db));
            match tree {
                Ok(tree) => {
                    if tree.root_node().has_error() {
                        ParseError::SyntaxError.report(db);
                        None
                    } else {
                        let mut context = ParseContext::new(db, input.path(db), input.root(db), buffer);
                        let root_id = #program_id::orphaned(&mut context, tree.root_node())
                        .map_or_else(|e| {
                            e.report(db);
                            None
                        }, |program| {
                            Some(program)
                        });
                        if let Some(program) = root_id {
                            Some(Parsed::new(db, context.file_id, Arc::new(context.tree), program))
                        } else {
                            None
                        }
                    }
                }
                Err(e) => {
                    e.report(db);
                    None
                }
            }
        }
        #[salsa::tracked(return_ref)]
        pub fn parse_program(db: &dyn salsa::Database, input: codegen_sdk_cst::File) -> Parsed<'_> {
            let raw = parse_program_raw(db, input);
            if let Some(parsed) = raw {
                parsed
            } else {
                panic!("Failed to parse program");
            }
        }
        pub struct #language_struct_name;
        impl CSTLanguage for #language_struct_name {
            type Types<'db> = NodeTypes<'db>;
            type Program<'db> = #root_node<'db>;
            fn language() -> &'static codegen_sdk_common::language::Language {
                &codegen_sdk_common::language::#language_name::#language_struct_name
            }
            fn parse<'db>(db: &'db dyn salsa::Database, content: std::string::String) -> Option<(&'db Self::Program<'db>, &'db Tree<Self::Types<'db>>, indextree::NodeId)> {
                let input = codegen_sdk_cst::File::new(db, std::path::PathBuf::new(), content, std::path::PathBuf::new());
                let parsed = parse_program(db, input);
                let program_id = parsed.program(db);
                let tree = parsed.tree(db);
                let program = tree.get(&program_id).unwrap().as_ref();
                Some((program.try_into().unwrap(), tree, program_id))
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
