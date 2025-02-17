pub use codegen_sdk_common::language::LANGUAGES;
fn main() {
    env_logger::init();
    for language in LANGUAGES.iter() {
        println!("{}", language.tag_query);
        for (name, query) in language.queries() {
            println!("{}", name);
            println!("{:?}", query);
        }
        for (name, query) in language.definitions() {
            println!("{}", name);
            println!("{:?}", query);
        }
        for (name, query) in language.references() {
            println!("{}", name);
            println!("{:?}", query);
        }
    }
}
