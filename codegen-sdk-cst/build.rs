use codegen_sdk_common::language::LANGUAGES;
use codegen_sdk_cst_generator::generate_cst;
use rayon::prelude::*;
fn main() {
    env_logger::init();
    println!("cargo:rerun-if-changed=build.rs");
    LANGUAGES.par_iter().for_each(|language| {
        generate_cst(language).unwrap_or_else(|e| {
            log::error!("Error generating CST for {}: {}", language.name, e);
        });
    });
}
