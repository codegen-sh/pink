use codegen_sdk_common::language::LANGUAGES;
use codegen_sdk_cst_generator::{Config, generate_cst_to_file};
use rayon::prelude::*;
fn main() {
    let config = Config {
        serialize: cfg!(feature = "serialization"),
    };
    env_logger::init();
    // println!("cargo:rerun-if-changed=build.rs");
    LANGUAGES.par_iter().for_each(|language| {
        generate_cst_to_file(language, config.clone()).unwrap_or_else(|e| {
            log::error!("Error generating CST for {}: {}", language.name(), e);
            panic!("Error generating CST for {}: {}", language.name(), e);
        });
    });
}
