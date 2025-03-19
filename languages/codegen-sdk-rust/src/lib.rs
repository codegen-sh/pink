#![recursion_limit = "2048"]
#![allow(unused, irrefutable_let_patterns)]
#![allow(non_snake_case)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/rust.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/rust-ast.rs"));
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for Symbol<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = RustStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            match self {
                _ => vec![RustStack::start(db, self)],
            }
        }
    }
    #[salsa::tracked]
    impl<'db> ResolveType<'db> for crate::ast::Reference<'db> {
        type Type = crate::ast::Symbol<'db>;
        type Stack = RustStack<'db>;
        #[salsa::tracked(return_ref)]
        fn resolve_type(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<Self::Stack> {
            match self {
                _ => vec![],
            }
        }
    }
}
