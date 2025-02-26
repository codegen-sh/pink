extern crate proc_macro;
use codegen_sdk_common::language::{LANGUAGES, Language};
use proc_macro::TokenStream;
fn get_language(language: &str) -> &Language {
    for lang in LANGUAGES.iter() {
        if lang.name().to_lowercase() == language.to_lowercase() {
            return lang;
        }
    }
    panic!("Language not found");
}
#[proc_macro]
pub fn include_language_ast(_item: TokenStream) -> TokenStream {
    let target_language = _item.to_string();
    let language = get_language(&target_language);

    format!(
        "#[cfg(feature = \"{name}\")]
pub mod {name} {{
    use codegen_sdk_cst::{name};
    include!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"));
}}",
        name = language.name()
    )
    .parse()
    .unwrap()
}

#[proc_macro]
pub fn include_language(_item: TokenStream) -> TokenStream {
    let target_language = _item.to_string();
    let language = get_language(&target_language);
    let root = language.root_node();

    format!(
        "#[cfg(feature = \"{name}\")]
pub mod {name} {{
    use crate::CSTLanguage;
    use codegen_sdk_common::language::Language;
    include!(concat!(env!(\"OUT_DIR\"), \"/{name}.rs\"));
    pub struct {struct_name};
    impl CSTLanguage for {struct_name} {{
        type Program = {root};
        fn language() -> &'static Language {{
            &codegen_sdk_common::language::{name}::{struct_name}
        }}
    }}
}}",
        name = language.name(),
        struct_name = language.struct_name,
        root = root
    )
    .parse()
    .unwrap()
}

#[proc_macro]
pub fn parse_language(_item: TokenStream) -> TokenStream {
    let target_language = _item.to_string();
    let language = get_language(&target_language);
    format!(
        "#[cfg(feature = \"{name}\")]
    if {name}::{struct_name}::should_parse(file_path)? {{
        let serialized_path = cache.get_path(file_path);
        if serialized_path.exists() {{
            log::debug!(\"Deserializing {name}\");
            let bytes = cache.read_entry(&serialized_path)?;
            let parsed =
                from_bytes::<<{name}::{struct_name} as CSTLanguage>::Program, rkyv::rancor::Error>(&bytes)?;
            return Ok(Box::new(parsed));
        }}
        let parsed = {name}::{struct_name}::parse_file(db, file_path)?;
        log::debug!(\"Serializing {name}\");
        let writer = cache.get_writer(&serialized_path)?;
        let _ = to_bytes_in::<_, rkyv::rancor::Error>(&parsed, writer)?;
        return Ok(Box::new(parsed));
    }}
 ",
        name = language.name(),
        struct_name = language.struct_name
    )
    .parse()
    .unwrap()
}
#[proc_macro]
pub fn parse_languages(_item: TokenStream) -> TokenStream {
    let mut output = String::new();
    output.push_str("use codegen_sdk_macros::parse_language;");
    for language in LANGUAGES.iter() {
        output.push_str(&format!("parse_language!({});", language.name()));
    }
    output.parse().unwrap()
}
#[proc_macro]
pub fn include_languages(_item: TokenStream) -> TokenStream {
    let mut output = String::new();
    output.push_str("use codegen_sdk_macros::include_language;");
    for language in LANGUAGES.iter() {
        output.push_str(&format!("include_language!({});", language.name()));
    }
    output.parse().unwrap()
}
#[proc_macro]
pub fn include_languages_ast(_item: TokenStream) -> TokenStream {
    let mut output = String::new();
    output.push_str("use codegen_sdk_macros::include_language_ast;");
    for language in LANGUAGES.iter() {
        output.push_str(&format!("include_language_ast!({});", language.name()));
    }
    output.parse().unwrap()
}
