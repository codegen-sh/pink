use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse_quote;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    pub name: String,
    pub kind: String,
    pub is_optional: bool,
    pub is_subenum: bool,
    pub is_multiple: bool,
    pub query: String,
}
impl Field {
    pub fn as_syn_field(&self) -> syn::Field {
        let doc = self.doc();
        let name_ident = syn::Ident::new(format!("_{}", &self.name).as_str(), Span::call_site());
        parse_quote!(
            #[doc = #doc]
            #[tracked]
            #[return_ref]
            pub #name_ident: codegen_sdk_common::CSTNodeTreeId
        )
    }
    fn doc(&self) -> String {
        format!("@{} from query: {:#?}", self.name, self.query)
    }
    fn type_name(&self) -> TokenStream {
        if self.is_subenum {
            let type_name = format_ident!("{}Ref", &self.kind);
            quote! {
                crate::cst::#type_name<'db>
            }
        } else {
            let type_name: syn::Ident = syn::Ident::new(&self.kind, Span::call_site());
            quote! {
                &'db crate::cst::#type_name<'db>
            }
        }
    }
    pub fn getter(&self) -> syn::Stmt {
        let name_ident = syn::Ident::new(&self.name, Span::call_site());
        let field_ident = syn::Ident::new(format!("_{}", &self.name).as_str(), Span::call_site());
        let msg = self.doc();
        let type_name = self.type_name();
        parse_quote!(
            pub fn #name_ident(&self, db: &'db dyn codegen_sdk_resolution::Db) -> #type_name {
                #[doc = #msg]
                let id = self.#field_ident(db).id(db);
                let file = self.file(db);
                let node = file.tree(db).get(&id).unwrap();
                node.as_ref().try_into().unwrap()
            }
        )
    }
}
