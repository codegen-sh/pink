use super::Language;

lazy_static! {
    pub static ref TSX: Language = Language::new(
        "tsx",
        "TSX",
        tree_sitter_typescript::TSX_NODE_TYPES,
        &["tsx"],
        tree_sitter_typescript::LANGUAGE_TSX.into(),
        tree_sitter_typescript::TAGS_QUERY,
    )
    .unwrap();
}
