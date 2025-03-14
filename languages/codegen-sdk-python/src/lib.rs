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
    pub struct PythonStack<'db> {
        #[tracked(return_ref)]
        data: Symbol<'db>,
        #[tracked(return_ref)]
        next: Option<PythonStack<'db>>,
    }
    #[salsa::tracked]
    impl<'db> codegen_sdk_resolution::ResolutionStack<'db, Symbol<'db>> for PythonStack<'db> {
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

    impl<'db> PythonStack<'db> {
        pub fn start(db: &'db dyn codegen_sdk_resolution::Db, data: Symbol<'db>) -> Self {
            Self::new(db, data, None)
        }
        pub fn push(self, db: &'db dyn codegen_sdk_resolution::Db, data: Symbol<'db>) -> Self {
            Self::new(db, data, Some(self))
        }
    }
    #[salsa::tracked]
    impl<'db> Import<'db> {
        #[salsa::tracked]
        fn resolve_import(
            self,
            db: &'db dyn codegen_sdk_resolution::Db,
        ) -> Option<codegen_sdk_common::FileNodeId> {
            let root_path = self.root_path(db);
            let module = self.module(db).source().replace(".", "/");
            let target_path = root_path.join(module).with_extension("py");
            log::info!(target: "resolution", "Resolving import to path: {:?}", target_path);
            target_path
                .canonicalize()
                .ok()
                .map(|path| codegen_sdk_common::FileNodeId::new(db, path))
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for Symbol<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = PythonStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            match self {
                Symbol::Import(import) => import.resolve_type(db).clone(),
                Symbol::Function(function) => vec![PythonStack::start(db, self)],
                Symbol::Class(class) => vec![PythonStack::start(db, self)],
                Symbol::Constant(constant) => constant.resolve_type(db).clone(),
            }
        }
    }
    #[salsa::tracked]
    pub struct PythonDependencies<'db> {
        #[id]
        id: codegen_sdk_common::FileNodeId,
        #[return_ref]
        #[tracked]
        #[no_eq]
        pub dependencies: codegen_sdk_common::hash::FxHashMap<
            codegen_sdk_resolution::FullyQualifiedName,
            codegen_sdk_common::hash::FxIndexSet<crate::ast::Call<'db>>,
        >,
    }
    impl<'db>
        codegen_sdk_resolution::Dependencies<
            'db,
            codegen_sdk_resolution::FullyQualifiedName,
            crate::ast::Call<'db>,
        > for PythonDependencies<'db>
    {
        fn get(
            &'db self,
            db: &'db dyn codegen_sdk_resolution::Db,
            key: &codegen_sdk_resolution::FullyQualifiedName,
        ) -> Option<&'db codegen_sdk_common::hash::FxIndexSet<crate::ast::Call<'db>>> {
            self.dependencies(db).get(key)
        }
    }
    #[salsa::tracked(return_ref, no_eq)]
    pub fn dependencies<'db>(
        db: &'db dyn codegen_sdk_resolution::Db,
        input: codegen_sdk_common::FileNodeId,
    ) -> PythonDependencies<'db> {
        let file = parse(db, input);
        PythonDependencies::new(db, file.id(db), file.compute_dependencies(db))
    }

    #[salsa::tracked(return_ref, no_eq)]
    pub fn dependency_matrix<'db>(
        db: &'db dyn codegen_sdk_resolution::Db,
    ) -> codegen_sdk_common::hash::FxIndexMap<
        codegen_sdk_resolution::FullyQualifiedName,
        codegen_sdk_common::hash::FxIndexSet<codegen_sdk_common::FileNodeId>,
    > {
        let mut ret: codegen_sdk_common::hash::FxIndexMap<
            codegen_sdk_resolution::FullyQualifiedName,
            codegen_sdk_common::hash::FxIndexSet<codegen_sdk_common::FileNodeId>,
        > = Default::default();
        let files = codegen_sdk_resolution::files(db);
        let dependencies: Vec<
            Vec<(
                codegen_sdk_resolution::FullyQualifiedName,
                codegen_sdk_common::FileNodeId,
            )>,
        > = salsa::par_map(db, files, |db, file| {
            let dependencies = dependencies(db, file.clone());
            let mut ret = Vec::default();
            for name in dependencies.dependencies(db).keys() {
                ret.push((name.clone(), file.clone()));
            }
            ret
        });
        for (name, dependencies) in dependencies.into_iter().flatten() {
            ret.entry(name).or_default().insert(dependencies);
        }
        ret
    }

    use codegen_sdk_resolution::ResolutionStack;

    #[salsa::tracked]
    impl<'db> Scope<'db> for PythonFile<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Dependencies = PythonDependencies<'db>;
        type ReferenceType = crate::ast::Call<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve(self, db: &'db dyn codegen_sdk_resolution::Db, name: String) -> Vec<Self::Type> {
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
        ) -> codegen_sdk_common::hash::FxHashMap<
            codegen_sdk_resolution::FullyQualifiedName,
            codegen_sdk_common::hash::FxIndexSet<Self::ReferenceType>,
        >
        where
            Self: 'db,
        {
            let mut dependencies: codegen_sdk_common::hash::FxHashMap<
                codegen_sdk_resolution::FullyQualifiedName,
                codegen_sdk_common::hash::FxIndexSet<Self::ReferenceType>,
            > = codegen_sdk_common::hash::FxHashMap::default();
            for reference in self.resolvables(db) {
                let resolved = reference.clone().resolve_type(db);
                for resolved in resolved.into_iter() {
                    for entry in resolved.entries(db) {
                        dependencies
                            .entry(entry.fully_qualified_name(db))
                            .or_default()
                            .insert(reference.clone());
                    }
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
        type Stack = PythonStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            let target_path = self.resolve_import(db);
            if let Some(target_path) = target_path {
                if let Some(_) = db.get_file_for_id(target_path) {
                    let file = PythonFile::parse(db, target_path);
                    let mut results = Vec::new();
                    for resolved in file.resolve(db, self.name(db).source()) {
                        for stack in resolved.resolve_type(db) {
                            results.push(stack.push(db, Symbol::Import(self.clone())));
                        }
                    }
                    return results;
                }
            }
            Vec::new()
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for crate::ast::Constant<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = PythonStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            // TODO: Implement assignment type resolution
            vec![PythonStack::start(db, crate::ast::Symbol::Constant(self))]
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for crate::ast::Call<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = PythonStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            let mut results = Vec::new();
            for resolved in self.resolve_definition_stack(db) {
                if let Symbol::Function(function) = resolved.bottom(db) {
                    // TODO: Implement function call return type resolution
                }
                results.push(*resolved);
            }
            results
        }
        #[salsa::tracked(return_ref)]
        fn resolve_definition_stack(
            self,
            db: &'db dyn codegen_sdk_resolution::Db,
        ) -> Vec<Self::Stack> {
            let scope = self.file(db);
            let tree = scope.node(db).unwrap().tree(db);
            let definitions = scope.resolve(db, self.node(db).function(tree).source());
            let mut results = Vec::new();
            for definition in definitions.into_iter() {
                results.extend(definition.resolve_type(db));
            }
            results
        }
    }
    use codegen_sdk_resolution::{Db, Dependencies, HasId};
    pub fn references_impl<'db>(
        db: &'db dyn Db,
        name: codegen_sdk_resolution::FullyQualifiedName,
    ) -> Vec<crate::ast::Call<'db>> {
        let mut results = Vec::new();
        let dependency_matrix = dependency_matrix(db);
        let files = dependency_matrix.get(&name);
        if let Some(files) = files {
            log::info!(target: "resolution", "Finding references across {:?} files", files.len());
            for input in files.into_iter() {
                let dependencies = dependencies(db, input.clone());
                if let Some(references) = dependencies.get(db, &name) {
                    results.extend(references.iter().cloned());
                }
            }
        }
        results
    }
    #[salsa::tracked]
    impl<'db>
        codegen_sdk_resolution::References<
            'db,
            PythonDependencies<'db>,
            crate::ast::Call<'db>,
            PythonFile<'db>,
        > for crate::ast::Symbol<'db>
    {
        fn references(self, db: &'db dyn Db) -> Vec<crate::ast::Call<'db>> {
            let mut results = Vec::new();
            for reference in references_impl(db, self.fully_qualified_name(db)).iter() {
                let resolved_stacks = reference.resolve_type(db);
                if resolved_stacks
                    .iter()
                    .any(|stack| stack.entries(db).iter().any(|entry| *entry == self))
                {
                    results.push(reference.clone());
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
