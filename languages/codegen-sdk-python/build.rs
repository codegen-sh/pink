use codegen_sdk_ast_generator::generate_ast;
use codegen_sdk_common::language::python::Python;
use codegen_sdk_cst_generator::{Config, generate_cst_to_file};

fn main() {
    let config = Config {
        serialize: cfg!(feature = "serialization"),
    };
    env_logger::init();
    generate_cst_to_file(&Python, config.clone()).unwrap_or_else(|e| {
        log::error!("Error generating CST for {}: {}", Python.name(), e);
        panic!("Error generating CST for {}: {}", Python.name(), e);
    });
    generate_ast(&Python).unwrap_or_else(|e| {
        log::error!("Error generating AST for {}: {}", Python.name(), e);
        panic!("Error generating AST for {}: {}", Python.name(), e);
    });
}
