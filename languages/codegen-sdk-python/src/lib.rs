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
    impl<'db> Import<'db> {
        #[salsa::tracked]
        fn resolve_import(self, db: &'db dyn codegen_sdk_resolution::Db) -> Option<PathBuf> {
            let root_path = self.root_path(db);
            let module = self.module(db).source().replace(".", "/");
            let target_path = root_path.join(module).with_extension("py");
            log::info!(target: "resolution", "Resolving import to path: {:?}", target_path);
            target_path.canonicalize().ok()
        }
    }
    #[salsa::tracked]
    impl<'db> Scope<'db> for PythonFile<'db> {
        type Type = crate::ast::Symbol<'db>;
        type ReferenceType = crate::ast::Call<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve(self, db: &'db dyn codegen_sdk_resolution::Db, name: String) -> Vec<Self::Type> {
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
                        for resolved in def.resolve_type(db) {
                            results.push(resolved.clone());
                        }
                    }
                }
            }
            results
        }
        #[salsa::tracked]
        fn resolvables(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::ReferenceType> {
            let mut results = Vec::new();
            for (_, refs) in self.references(db).calls(db).into_iter() {
                results.extend(refs.into_iter().cloned());
            }
            results
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for crate::ast::Import<'db> {
        type Type = crate::ast::Symbol<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Type> {
            let target_path = self.resolve_import(db);
            if let Some(target_path) = target_path {
                if let Some(input) = db.get_file(target_path) {
                    return PythonFile::parse(db, input, self.root_path(db))
                        .resolve(db, self.name(db).source())
                        .to_vec();
                }
            }
            Vec::new()
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for crate::ast::Call<'db> {
        type Type = crate::ast::Symbol<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Type> {
            let scope = self.file(db);
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
        fn filter(
            &self,
            db: &'db dyn codegen_sdk_resolution::Db,
            input: &codegen_sdk_ast::input::File,
        ) -> bool {
            match self {
                crate::ast::Symbol::Function(function) => {
                    let content = input.contents(db).content(db);
                    let target = function.name(db).text();
                    memchr::memmem::find(&content.as_bytes(), &target).is_some()
                }
                _ => true,
            }
        }
    }
}
