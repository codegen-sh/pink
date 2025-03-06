#![recursion_limit = "2048"]
#![allow(unused)]
#![allow(non_snake_case)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/python.rs"));
}
pub mod ast {
    use codegen_sdk_ast::{Definitions as _, References as _};
    use codegen_sdk_resolution::{ResolveType, Scope};
    include!(concat!(env!("OUT_DIR"), "/python-ast.rs"));
    #[salsa::tracked]
    impl<'db> Scope<'db> for PythonFile<'db> {
        type Type = crate::ast::Symbol<'db>;
        type ReferenceType = crate::ast::Call<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve(self, db: &'db dyn salsa::Database, name: String) -> Vec<Self::Type> {
            let tree = self.node(db).unwrap().tree(db);
            let mut results = Vec::new();
            for (def_name, defs) in self.definitions(db).functions(db).into_iter() {
                if *def_name == name {
                    results.extend(
                        defs.into_iter()
                            .cloned()
                            .map(|def| crate::ast::Symbol::Function(def)),
                    );
                }
            }
            for (def_name, defs) in self.definitions(db).imports(db).into_iter() {
                if *def_name == name {
                    results.extend(
                        defs.into_iter()
                            .cloned()
                            .map(|def| crate::ast::Symbol::Import(def)),
                    );
                }
            }
            results
        }
        #[salsa::tracked]
        fn resolvables(self, db: &'db dyn salsa::Database) -> Vec<Self::ReferenceType> {
            let mut results = Vec::new();
            for (_, refs) in self.references(db).calls(db).into_iter() {
                results.extend(refs.into_iter().cloned());
            }
            results
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db, PythonFile<'db>> for crate::ast::Call<'db> {
        type Type = crate::ast::Symbol<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(
            self,
            db: &'db dyn salsa::Database,
            scope: PythonFile<'db>,
            _scopes: Vec<PythonFile<'db>>,
        ) -> Vec<Self::Type> {
            let tree = scope.node(db).unwrap().tree(db);
            scope
                .resolve(db, self.node(db).function(tree).source())
                .clone()
        }
    }
    #[salsa::tracked]
    impl<'db> codegen_sdk_resolution::References<'db, crate::ast::Call<'db>, PythonFile<'db>>
        for crate::ast::Symbol<'db>
    {
    }
}
