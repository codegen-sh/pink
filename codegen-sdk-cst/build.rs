use codegen_sdk_common::language::LANGUAGES;
use codegen_sdk_cst_generator::generate_cst;

fn main() {
    for language in LANGUAGES.iter() {
        generate_cst(language).unwrap();
    }
}
