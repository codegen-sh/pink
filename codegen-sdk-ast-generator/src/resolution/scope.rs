use codegen_sdk_common::Language;
use syn::parse_quote;

use crate::resolution::dependencies::get_dependencies_name;

pub fn generate_scope(language: &Language) -> Vec<syn::Stmt> {
    let dependencies_name = get_dependencies_name(language);
    let file_name = language.file_struct_name();
    parse_quote!(
        #[salsa::tracked]
        impl<'db> codegen_sdk_resolution::Scope<'db> for #file_name<'db> {
            type Type = crate::ast::Symbol<'db>;
            type Dependencies = #dependencies_name<'db>;
            type ReferenceType = crate::ast::Reference<'db>;
            #[salsa::tracked(return_ref)]
            fn resolve(
                self,
                db: &'db dyn codegen_sdk_resolution::Db,
                name: String,
            ) -> Vec<Self::Type> {
                let node = match self.node(db) {
                    Some(node) => node,
                    None => {
                        log::warn!(target: "resolution", "No node found for file: {:?}", self.id(db));
                        return Vec::new();
                    }
                };
                let tree = node.tree(db);
                let mut results = Vec::new();
                if let Some(defs) = self.definitions(db).symbols(db).get(&name) {
                    if let Some(def) = defs.into_iter().rev().next() {
                        results.push(*def);
                    }
                }
                results
            }
            #[salsa::tracked]
            fn resolvables(
                self,
                db: &'db dyn codegen_sdk_resolution::Db,
            ) -> Vec<Self::ReferenceType> {
                let mut results = Vec::new();
                for (_, refs) in self.references(db).references(db).into_iter() {
                    results.extend(refs.into_iter().cloned());
                }
                results
            }

            #[salsa::tracked(return_ref)]
            fn compute_dependencies_query(
                self,
                db: &'db dyn codegen_sdk_resolution::Db,
            ) -> #dependencies_name<'db> {
                #dependencies_name::new(db, self.id(db), self.compute_dependencies(db))
            }
        }
    )
}
