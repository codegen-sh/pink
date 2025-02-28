use codegen_sdk_ast_generator::generate_ast;
use codegen_sdk_common::language::yaml::YAML;
use codegen_sdk_cst_generator::{Config, generate_cst_to_file};

fn main() {
    let config = Config {
        serialize: cfg!(feature = "serialization"),
    };
    env_logger::init();
    generate_cst_to_file(&YAML, config.clone()).unwrap_or_else(|e| {
        log::error!("Error generating CST for {}: {}", YAML.name(), e);
        panic!("Error generating CST for {}: {}", YAML.name(), e);
    });
    generate_ast(&YAML).unwrap_or_else(|e| {
        log::error!("Error generating AST for {}: {}", YAML.name(), e);
        panic!("Error generating AST for {}: {}", YAML.name(), e);
    });
}
