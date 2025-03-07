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
    pub struct PythonDependencies<'db> {
        #[id]
        id: codegen_sdk_common::FileNodeId<'db>,
        #[return_ref]
        #[tracked]
        dependencies: codegen_sdk_resolution::indexmap::IndexMap<
            codegen_sdk_resolution::FullyQualifiedName<'db>,
            codegen_sdk_resolution::indexmap::IndexSet<crate::ast::Call<'db>>,
        >,
    }
    impl<'db>
        codegen_sdk_resolution::Dependencies<
            'db,
            codegen_sdk_resolution::FullyQualifiedName<'db>,
            crate::ast::Call<'db>,
        > for PythonDependencies<'db>
    {
        fn get(
            &'db self,
            db: &'db dyn codegen_sdk_resolution::Db,
            key: &codegen_sdk_resolution::FullyQualifiedName<'db>,
        ) -> Option<&'db codegen_sdk_resolution::indexmap::IndexSet<crate::ast::Call<'db>>>
        {
            self.dependencies(db).get(key)
        }
    }
    #[salsa::tracked(return_ref)]
    pub fn dependencies(
        db: &dyn codegen_sdk_resolution::Db,
        input: codegen_sdk_cst::File,
    ) -> PythonDependencies<'_> {
        let file = parse(db, input);
        PythonDependencies::new(db, file.id(db), file.compute_dependencies(db))
    }
    #[salsa::tracked]
    impl<'db> Scope<'db> for PythonFile<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Dependencies = PythonDependencies<'db>;
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
        fn compute_dependencies(
            self,
            db: &'db dyn codegen_sdk_resolution::Db,
        ) -> codegen_sdk_resolution::indexmap::IndexMap<
            codegen_sdk_resolution::FullyQualifiedName<'db>,
            codegen_sdk_resolution::indexmap::IndexSet<Self::ReferenceType>,
        >
        where
            Self: 'db,
        {
            let mut dependencies: codegen_sdk_resolution::indexmap::IndexMap<
                codegen_sdk_resolution::FullyQualifiedName<'db>,
                codegen_sdk_resolution::indexmap::IndexSet<Self::ReferenceType>,
            > = codegen_sdk_resolution::indexmap::IndexMap::new();
            for reference in self.resolvables(db) {
                let resolved = reference.clone().resolve_type(db);
                for resolved in resolved {
                    dependencies
                        .entry(resolved.fully_qualified_name(db))
                        .or_default()
                        .insert(reference.clone());
                }
            }
            dependencies
        }

        #[salsa::tracked(return_ref)]
        fn compute_dependencies_query(
            self,
            db: &'db dyn codegen_sdk_resolution::Db,
        ) -> PythonDependencies<'db> {
            PythonDependencies::new(db, self.id(db), self.compute_dependencies(db))
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
                    return PythonFile::parse(db, input)
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
    use codegen_sdk_resolution::{Db, Dependencies, HasId};
    #[salsa::tracked]
    impl<'db>
        codegen_sdk_resolution::References<
            'db,
            PythonDependencies<'db>,
            crate::ast::Call<'db>,
            PythonFile<'db>,
        > for crate::ast::Symbol<'db>
    {
        fn references(&self, db: &'db dyn Db) -> Vec<crate::ast::Call<'db>> {
            let files = codegen_sdk_resolution::files(db);
            log::info!(target: "resolution", "Finding references across {:?} files", files.len());
            let mut results = Vec::new();
            let name = self.fully_qualified_name(db);
            for input in files {
                // if !self.filter(db, &input) {
                //     continue;
                // }
                let dependencies = dependencies(db, input.clone());
                if let Some(references) = dependencies.get(db, &name) {
                    results.extend(references.iter().cloned());
                }
            }
            results
        }
        fn filter(
            &self,
            db: &'db dyn codegen_sdk_resolution::Db,
            input: &codegen_sdk_cst::File,
        ) -> bool {
            match self {
                crate::ast::Symbol::Function(function) => {
                    let content = input.content(db);
                    let target = function.name(db).text();
                    memchr::memmem::find(&content.as_bytes(), &target).is_some()
                }
                _ => true,
            }
        }
    }
}
