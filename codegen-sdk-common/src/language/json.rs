use super::Language;
lazy_static! {
    pub static ref JSON: Language = Language {
        name: "json",
        struct_name: "JSON",
        node_types: tree_sitter_json::NODE_TYPES,
        file_extensions: &["json"],
        tree_sitter_language: tree_sitter_json::LANGUAGE.into(),
    };
}
