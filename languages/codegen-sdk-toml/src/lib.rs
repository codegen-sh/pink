#![recursion_limit = "2048"]
#![allow(unused)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/toml.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/toml-ast.rs"));
}
