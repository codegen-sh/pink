use codegen_sdk_common::language::Language;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
fn get_definitions_impl(language: &Language) -> TokenStream {
    let language_struct_name = format_ident!("{}File", language.struct_name);
    if !language.tag_query.contains("@definition") {
        return quote! {

        impl<'db> codegen_sdk_ast::Definitions<'db> for #language_struct_name<'db> {
            type Definitions = ();
            fn definitions(self, _db: &'db dyn salsa::Database) -> &'db Self::Definitions{
                &()
            }
        }
        };
    }
    quote! {
        #[salsa::tracked]
        impl<'db> codegen_sdk_ast::Definitions<'db> for #language_struct_name<'db> {
            type Definitions = Definitions<'db>;
            #[salsa::tracked(return_ref)]
            fn definitions(self, db: &'db dyn salsa::Database) -> Self::Definitions {
                if let Some(program) = self.node(db) {
                    return Definitions::visit(db, program);
                } else {
                    return Definitions::default(db);
                }
            }
        }
    }
}
fn get_references_impl(language: &Language) -> TokenStream {
    let language_struct_name = format_ident!("{}File", language.struct_name);
    if !language.tag_query.contains("@reference") {
        return quote! {
            impl<'db> codegen_sdk_ast::References<'db> for #language_struct_name<'db> {
                type References = ();
                fn references(self, _db: &'db dyn salsa::Database) -> &'db Self::References {
                    &()
                }
            }
        };
    }
    quote! {
        #[salsa::tracked]
        impl<'db> codegen_sdk_ast::References<'db> for #language_struct_name<'db> {
            type References = References<'db>;
            #[salsa::tracked(return_ref)]
            fn references(self, db: &'db dyn salsa::Database) -> Self::References {
                if let Some(program) = self.node(db) {
                    return References::visit(db, program);
                } else {
                    return References::default(db);
                }
            }
        }
    }
}
pub fn generate_ast(language: &Language) -> anyhow::Result<TokenStream> {
    let language_struct_name = format_ident!("{}File", language.struct_name);
    let language_name_str = language.name();
    let definitions_impl = get_definitions_impl(language);
    let references_impl = get_references_impl(language);
    let root_node_name = format_ident!("{}", language.root_node());
    let content = quote! {
    #[salsa::tracked]
    pub struct #language_struct_name<'db> {
        #[tracked]
        #[return_ref]
        pub node: Option<crate::cst::Parsed<'db>>,
        #[id]
        pub id: codegen_sdk_common::FileNodeId,
    }
    impl<'db> codegen_sdk_resolution::Parse<'db> for #language_struct_name<'db> {
        fn parse(db: &'db dyn codegen_sdk_resolution::Db, input: codegen_sdk_common::FileNodeId) -> &'db Self {
            parse(db, input)
        }
    }
    impl<'db> codegen_sdk_resolution::File<'db> for #language_struct_name<'db> {
        fn path(&self, db: &'db dyn salsa::Database) -> &PathBuf {
            &self.id(db).path(db)
        }
    }
    #[salsa::tracked(return_ref)]
    pub fn parse<'db>(db: &'db dyn codegen_sdk_resolution::Db, input: codegen_sdk_common::FileNodeId) -> #language_struct_name<'db> {
        let input = db.input(input.path(db)).unwrap();
        log::debug!("Parsing {} file: {}", input.path(db).display(), #language_name_str);
        let ast = crate::cst::parse_program_raw(db, input);
        let file_id = codegen_sdk_common::FileNodeId::new(db, input.path(db).clone());
        #language_struct_name::new(db, ast, file_id)
    }
    impl<'db> #language_struct_name<'db> {
        pub fn tree(&self, db: &'db dyn salsa::Database) -> &'db codegen_sdk_common::Tree<crate::cst::NodeTypes<'db>> {
            self.node(db).unwrap().tree(db)
        }
        pub fn root(&self, db: &'db dyn salsa::Database) -> &'db crate::cst::#root_node_name<'db> {
            let tree = self.tree(db);
            tree.get(&self.node(db).unwrap().program(db)).unwrap().as_ref().try_into().unwrap()
        }
    }
    #definitions_impl
    #references_impl
    // impl<'db> HasNode for {language_struct_name}File<'db> {
    //     type Node = {language_name}::{root_node_name}<'db>;
    //     fn node(&self) -> &Self::Node {
    //         &self.node
    //     }
    // }

    };

    Ok(content)
}
