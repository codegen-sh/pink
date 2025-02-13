use super::Language;

lazy_static! {
    pub static ref Javascript: Language = Language {
        name: "javascript",
        node_types: tree_sitter_javascript::NODE_TYPES,
        file_extensions: &["js"],
        tree_sitter_language: tree_sitter_javascript::LANGUAGE.into(),
    };
}
