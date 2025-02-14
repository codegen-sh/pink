use codegen_sdk_common::language::LANGUAGES;
use codegen_sdk_cst_generator::generate_cst;

fn main() {
    env_logger::init();
    for language in LANGUAGES.iter() {
        generate_cst(language).unwrap();
    }
}
