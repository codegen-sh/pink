use codegen_sdk_ast_generator::generate_ast;
use codegen_sdk_common::language::rust::Rust;
use codegen_sdk_cst_generator::{Config, generate_cst_to_file};

fn main() {
    let config = Config {
        serialize: cfg!(feature = "serialization"),
    };
    env_logger::init();
    generate_cst_to_file(&Rust, config.clone()).unwrap_or_else(|e| {
        log::error!("Error generating CST for {}: {}", Rust.name(), e);
        panic!("Error generating CST for {}: {}", Rust.name(), e);
    });
    generate_ast(&Rust).unwrap_or_else(|e| {
        log::error!("Error generating AST for {}: {}", Rust.name(), e);
        panic!("Error generating AST for {}: {}", Rust.name(), e);
    });
}
