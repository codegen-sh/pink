use codegen_sdk_ast_generator::generate_ast;
use codegen_sdk_common::language::javascript::JavaScript;
use codegen_sdk_cst_generator::{Config, generate_cst_to_file};

fn main() {
    let config = Config {
        serialize: cfg!(feature = "serialization"),
    };
    env_logger::init();
    generate_cst_to_file(&JavaScript, config.clone()).unwrap_or_else(|e| {
        log::error!("Error generating CST for {}: {}", JavaScript.name(), e);
        panic!("Error generating CST for {}: {}", JavaScript.name(), e);
    });
    generate_ast(&JavaScript).unwrap_or_else(|e| {
        log::error!("Error generating AST for {}: {}", JavaScript.name(), e);
        panic!("Error generating AST for {}: {}", JavaScript.name(), e);
    });
}
