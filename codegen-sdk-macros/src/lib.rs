extern crate proc_macro;
use codegen_sdk_common::language::{Language, LANGUAGES};
use codegen_sdk_cst_generator::parser::parse_node_types;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
fn get_language(language: &str) -> &Language {
    for lang in LANGUAGES.iter() {
        if lang.name.to_lowercase() == language.to_lowercase() {
            return lang;
        }
    }
    panic!("Language not found");
}
#[proc_macro]
pub fn include_language(_item: TokenStream) -> TokenStream {
    let target_language = _item.to_string();
    let language = get_language(&target_language);
    let nodes = parse_node_types(&language).unwrap();
    let mut root = "Program".to_string();
    for node in nodes {
        if node.root {
            root = node.type_name.to_string();
        }
    }

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
        name = language.name,
        struct_name = language.struct_name,
        root = root.to_case(Case::Pascal)
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
        let parsed = {name}::{struct_name}::parse_file(file_path)?;
        return Ok(Box::new(parsed));
    }}
 ",
        name = language.name,
        struct_name = language.struct_name
    )
    .parse()
    .unwrap()
}
