use super::Language;
lazy_static! {
    pub static ref JSX: Language = Language {
        name: "jsx",
        node_types: tree_sitter_typescript::TSX_NODE_TYPES,
        file_extensions: &["jsx"],
        tree_sitter_language: tree_sitter_typescript::LANGUAGE_TSX.into(),
    };
}
