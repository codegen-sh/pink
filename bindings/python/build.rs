use codegen_bindings_generator::{generate_python_bindings, generate_python_bindings_common};
use codegen_sdk_common::{Language, language::LANGUAGES};
fn main() {
    env_logger::init();
    let languages: Vec<&Language> = LANGUAGES
        .iter()
        .filter(|language| language.name() != "ts_query")
        .cloned()
        .collect();
    for language in languages.iter() {
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
    generate_python_bindings_common(&languages).unwrap();
}
