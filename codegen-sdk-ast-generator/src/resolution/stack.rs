// Generates the resolution stack for a language
// The reason we need to do this rather than use generics is that Salsa isn't very friendly with generics.
use codegen_sdk_common::Language;
use quote::format_ident;
use syn::parse_quote;
pub fn get_stack_name(language: &Language) -> syn::Ident {
    format_ident!("{}Stack", language.struct_name())
}
pub fn generate_stack(language: &Language) -> Vec<syn::Stmt> {
    let stack_name = get_stack_name(language);
    parse_quote! {
        #[salsa::tracked]
        pub struct #stack_name<'db> {
            #[tracked(return_ref)]
            data: Symbol<'db>,
            #[tracked(return_ref)]
            next: Option<#stack_name<'db>>,
        }
        #[salsa::tracked]
        impl<'db> codegen_sdk_resolution::ResolutionStack<'db, Symbol<'db>> for #stack_name<'db> {
            #[salsa::tracked(return_ref)]
            fn bottom(self, db: &'db dyn codegen_sdk_resolution::Db) -> Symbol<'db> {
                match self.next(db) {
                    Some(next) => *next.bottom(db),
                    None => self.data(db),
                }
            }
            #[salsa::tracked(return_ref)]
            fn entries(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Symbol<'db>> {
                match &self.next(db) {
                    Some(next) => {
                        let mut entries = next.entries(db).clone();
                        entries.push(self.data(db));
                        entries
                    }
                    None => vec![self.data(db)],
                }
            }
        }
        impl<'db> #stack_name<'db> {
            pub fn start(db: &'db dyn codegen_sdk_resolution::Db, data: Symbol<'db>) -> Self {
                Self::new(db, data, None)
            }
            pub fn push(self, db: &'db dyn codegen_sdk_resolution::Db, data: Symbol<'db>) -> Self {
                Self::new(db, data, Some(self))
            }
        }
    }
}
