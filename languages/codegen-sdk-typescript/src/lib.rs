#![recursion_limit = "2048"]
#![allow(non_snake_case)]
#![allow(unused)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/typescript.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/typescript-ast.rs"));
}
