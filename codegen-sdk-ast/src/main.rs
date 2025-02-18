pub use codegen_sdk_common::language::LANGUAGES;
fn main() {
    for language in LANGUAGES.iter() {
        println!("{}", language.tags_query);
    }
}
