use codegen_sdk_common::language::Language;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
fn get_definitions_impl(language: &Language) -> TokenStream {
    if !language.tag_query.contains("@definition") {
        return quote! {};
    }
    quote! {
        #[salsa::tracked]
        pub fn definitions(self, db: &'db dyn salsa::Database) -> Definitions<'db> {
            let mut definitions = Definitions::default();
            if let Some(program) = self.node(db) {
                definitions = definitions.visit_by_val_infallible(&program);
            }
            definitions
        }
    }
}
fn get_references_impl(language: &Language) -> TokenStream {
    if !language.tag_query.contains("@reference") {
        return quote! {};
    }
    quote! {
        #[salsa::tracked]
        pub fn references(self, db: &'db dyn salsa::Database) -> References<'db> {
            let mut references = References::default();
            if let Some(program) = self.node(db) {
                references = references.visit_by_val_infallible(&program);
            }
            references
        }
    }
}
pub fn generate_ast(language: &Language) -> anyhow::Result<TokenStream> {
    let language_struct_name = format_ident!("{}File", language.struct_name);
    let language_name_str = language.name();
    let definitions_impl = get_definitions_impl(language);
    let references_impl = get_references_impl(language);
    let program_id = format_ident!("{}", language.root_node());
    let content = quote! {
    #[salsa::tracked]
    pub struct #language_struct_name<'db> {
        #[return_ref]
        node: Option<crate::cst::#program_id<'db>>,
        #[id]
        pub path: PathBuf,
    }
    // impl<'db> File for {language_struct_name}File<'db> {{
    //     fn path(&self) -> &PathBuf {{
    //         &self.path(db)
    //     }}
    // }}
    pub fn parse(db: &dyn salsa::Database, input: codegen_sdk_ast::input::File) -> #language_struct_name<'_> {
        log::debug!("Parsing {} file: {}", input.path(db).display(), #language_name_str);
        let ast = crate::cst::parse_program_raw(db, input.contents(db));
        #language_struct_name::new(db, ast, input.path(db).clone())
    }
    #[salsa::tracked]
    pub fn parse_query(db: &dyn salsa::Database, input: codegen_sdk_ast::input::File) -> #language_struct_name<'_> {
        parse(db, input)
    }


    #[salsa::tracked]
    impl<'db> #language_struct_name<'db> {
        #definitions_impl
        #references_impl
    }
    // impl<'db> HasNode for {language_struct_name}File<'db> {
    //     type Node = {language_name}::{root_node_name}<'db>;
    //     fn node(&self) -> &Self::Node {
    //         &self.node
    //     }
    // }

    };

    Ok(content)
}
