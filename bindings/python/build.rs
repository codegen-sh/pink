use codegen_bindings_generator::{generate_python_bindings, generate_python_bindings_common};
use codegen_sdk_common::language::LANGUAGES;
fn main() {
    env_logger::init();
    for language in LANGUAGES.iter() {
        if language.name() == "ts_query" {
            continue;
        }
        generate_python_bindings(&language).unwrap_or_else(|e| {
            log::error!(
                "Error generating Python bindings for {}: {}",
                language.name(),
                e
            );
            panic!(
                "Error generating Python bindings for {}: {}",
                language.name(),
                e
            );
        });
    }
    generate_python_bindings_common(&LANGUAGES).unwrap();
}
