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
        let name_ident = syn::Ident::new(&self.name, Span::call_site());
        let type_name = syn::Ident::new(&self.kind, Span::call_site());
        parse_quote!(
            #[tracked]
            #[return_ref]
            pub #name_ident: crate::cst::#type_name<'db>
        )
    }
}
