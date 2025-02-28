#![recursion_limit = "2048"]
#![allow(unused)]
#![allow(non_snake_case)]

pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/jsx.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/jsx-ast.rs"));
}
