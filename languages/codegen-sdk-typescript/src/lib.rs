#![recursion_limit = "2048"]
#![allow(non_snake_case)]
#![allow(unused)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/typescript.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/typescript-ast.rs"));
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for Symbol<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = TypescriptStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            match self {
                _ => vec![TypescriptStack::start(db, self)],
            }
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for crate::ast::Reference<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = TypescriptStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            match self {
                _ => vec![],
            }
        }
    }
}
