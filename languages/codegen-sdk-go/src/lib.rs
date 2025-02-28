#![recursion_limit = "2048"]
#![allow(unused)]
mod cst {
    include!(concat!(env!("OUT_DIR"), "/go.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/go-ast.rs"));
}
