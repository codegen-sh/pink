#![recursion_limit = "2048"]
#![allow(unused)]
#![allow(non_snake_case)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/python.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/python-ast.rs"));
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
    use codegen_sdk_resolution::{Db, Dependencies};
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for crate::ast::Reference<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = PythonStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            match self {
                crate::ast::Reference::Call(call) => call.resolve_type(db).clone(),
                _ => vec![],
            }
        }
    }
}
