use codegen_sdk_ast_generator::generate_ast;
use codegen_sdk_common::language::LANGUAGES;
use rayon::prelude::*;
fn main() {
    env_logger::init();
    LANGUAGES.iter().for_each(|language| {
        generate_ast(language).unwrap();
    });
}
