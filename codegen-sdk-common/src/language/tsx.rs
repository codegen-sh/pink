use super::Language;

lazy_static! {
    pub static ref TSX: Language = Language {
        name: "tsx",
        struct_name: "TSX",
        node_types: tree_sitter_typescript::TSX_NODE_TYPES,
        file_extensions: &["tsx"],
        tree_sitter_language: tree_sitter_typescript::LANGUAGE_TSX.into(),
    };
}
