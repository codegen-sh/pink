#![recursion_limit = "2048"]
#![allow(unused)]
#![allow(non_snake_case)]
mod cst {
    include!(concat!(env!("OUT_DIR"), "/rust.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/rust-ast.rs"));
}
