use proc_macro2::Span;
use syn::parse_quote_spanned;

use crate::query::field::Field;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub type_name: String,
    pub language_struct: String,
    pub fields: Vec<Field>,
}
impl Symbol {
    pub fn as_syn_struct(&self) -> Vec<syn::Stmt> {
        let span = Span::call_site();
        let variant = syn::Ident::new(&self.name, span);
        let type_name = syn::Ident::new(&self.type_name, span);
        let language_struct = syn::Ident::new(&self.language_struct, span);
        let fields = self
            .fields
            .iter()
            .map(|field| field.as_syn_field())
            .collect::<Vec<_>>();
        parse_quote_spanned! {
            span =>
            #[salsa::tracked]
            pub struct #variant<'db> {
                #[id]
                _fully_qualified_name: codegen_sdk_resolution::FullyQualifiedName,
                #[id]
                node_id: indextree::NodeId,
                // #[tracked]
                // #[return_ref]
                // pub node: crate::cst::#type_name<'db>,
                #(#fields),*
            }
            impl<'db> #variant<'db> {
                pub fn node(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &'db crate::cst::#type_name<'db> {
                    let file = self.file(db);
                    let tree = file.tree(db);
                    tree.get(&self.node_id(db)).unwrap().as_ref().try_into().unwrap()
                }
            }
            impl<'db> codegen_sdk_resolution::HasFile<'db> for #variant<'db> {
                type File<'db1> = #language_struct<'db1>;
                fn file(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &'db Self::File<'db> {
                    let path = self._fully_qualified_name(db).path(db);
                    parse(db, path)
                }
                fn root_path(&self, db: &'db dyn codegen_sdk_resolution::Db) -> PathBuf {
                    self.node(db).id().root(db).path(db)
                }
            }
            impl<'db> codegen_sdk_resolution::HasId<'db> for #variant<'db> {
                fn fully_qualified_name(&self, db: &'db dyn salsa::Database) -> codegen_sdk_resolution::FullyQualifiedName {
                    self._fully_qualified_name(db)
                }
            }
        }
    }
}
