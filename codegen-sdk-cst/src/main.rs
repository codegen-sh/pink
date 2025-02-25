use codegen_sdk_common::language::LANGUAGES;
use codegen_sdk_cst::query::HasQuery;
fn main() {
    for language in LANGUAGES.iter() {
        println!("{}", language.name());
        println!("{:#?}", language.queries());
        println!("{:#?}", language.definitions());
        println!("{:#?}", language.references());
    }
}
