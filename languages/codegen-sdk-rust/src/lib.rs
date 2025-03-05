#![recursion_limit = "2048"]
#![allow(unused, irrefutable_let_patterns)]
#![allow(non_snake_case)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/rust.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/rust-ast.rs"));
}
