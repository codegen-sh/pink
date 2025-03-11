use proc_macro2::Span;
use syn::parse_quote;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    pub name: String,
    pub kind: String,
    pub is_optional: bool,
    pub is_multiple: bool,
}
impl Field {
    pub fn as_syn_field(&self) -> syn::Field {
        let name_ident = syn::Ident::new(format!("_{}", &self.name).as_str(), Span::call_site());
        parse_quote!(
            #[tracked]
            #[return_ref]
            pub #name_ident: indextree::NodeId
        )
    }
    pub fn getter(&self) -> syn::Stmt {
        let name_ident = syn::Ident::new(&self.name, Span::call_site());
        let field_ident = syn::Ident::new(format!("_{}", &self.name).as_str(), Span::call_site());
        let type_name = syn::Ident::new(&self.kind, Span::call_site());
        parse_quote!(
            pub fn #name_ident(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &'db crate::cst::#type_name<'db> {
                let id = self.#field_ident(db);
                let file = self.file(db);
                let node = file.tree(db).get(&id).unwrap();
                node.as_ref().try_into().unwrap()
            }
        )
    }
}
