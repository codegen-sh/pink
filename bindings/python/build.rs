use codegen_bindings_generator::generate_python_bindings;
use codegen_sdk_common::language::LANGUAGES;
fn main() {
    env_logger::init();
    for language in LANGUAGES.iter() {
        generate_python_bindings(&language).unwrap_or_else(|e| {
            log::error!("Error generating CST for {}: {}", language.name(), e);
            panic!("Error generating CST for {}: {}", language.name(), e);
        });
    }
}
