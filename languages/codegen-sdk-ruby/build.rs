use codegen_sdk_ast_generator::generate_ast;
use codegen_sdk_common::language::ruby::Ruby;
use codegen_sdk_cst_generator::{Config, generate_cst_to_file};

fn main() {
    let config = Config {
        serialize: cfg!(feature = "serialization"),
    };
    env_logger::init();
    generate_cst_to_file(&Ruby, config.clone()).unwrap_or_else(|e| {
        log::error!("Error generating CST for {}: {}", Ruby.name(), e);
        panic!("Error generating CST for {}: {}", Ruby.name(), e);
    });
    generate_ast(&Ruby).unwrap_or_else(|e| {
        log::error!("Error generating AST for {}: {}", Ruby.name(), e);
        panic!("Error generating AST for {}: {}", Ruby.name(), e);
    });
}
