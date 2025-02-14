use codegen_sdk_common::language::LANGUAGES;
use codegen_sdk_cst_generator::generate_cst;
use rayon::prelude::*;
fn main() {
    env_logger::init();
    LANGUAGES.par_iter().for_each(|language| {
        generate_cst(language).unwrap();
    });
}
