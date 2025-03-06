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
        fn resolve(
            self,
            db: &'db dyn salsa::Database,
            name: String,
            root_path: PathBuf,
            scopes: Vec<PythonFile<'db>>,
        ) -> Vec<Self::Type> {
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
                    for def in defs {
                        results.push(crate::ast::Symbol::Import(def.clone()));
                        for resolved in
                            def.resolve_type(db, self, root_path.clone(), scopes.clone())
                        {
                            results.push(resolved.clone());
                        }
                    }
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
    impl<'db> ResolveType<'db, PythonFile<'db>> for crate::ast::Import<'db> {
        type Type = crate::ast::Symbol<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(
            self,
            db: &'db dyn salsa::Database,
            scope: PythonFile<'db>,
            root_path: PathBuf,
            scopes: Vec<PythonFile<'db>>,
        ) -> Vec<Self::Type> {
            let module = self.module(db).source().replace(".", "/");
            let target_path = FileNodeId::new(db, root_path.join(module).with_extension("py"));
            log::info!("Target path: {:?}", target_path);
            let name = self.name(db).source();
            for scope in &scopes {
                log::info!("Checking scope {:?}", scope.id(db));
                if scope.id(db) == target_path {
                    return scope.resolve(db, name, root_path, scopes).to_vec();
                }
            }
            Vec::new()
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
            root_path: PathBuf,
            scopes: Vec<PythonFile<'db>>,
        ) -> Vec<Self::Type> {
            log::info!("Resolving call with {:?} scopes", scopes.len());
            let tree = scope.node(db).unwrap().tree(db);
            scope
                .resolve(db, self.node(db).function(tree).source(), root_path, scopes)
                .clone()
        }
    }
    #[salsa::tracked]
    impl<'db> codegen_sdk_resolution::References<'db, crate::ast::Call<'db>, PythonFile<'db>>
        for crate::ast::Symbol<'db>
    {
    }
}
