use super::Language;
lazy_static! {
    pub static ref JSX: Language = Language::new(
        "jsx",
        "JSX",
        tree_sitter_typescript::TSX_NODE_TYPES,
        &["jsx"],
        tree_sitter_typescript::LANGUAGE_TSX.into(),
        tree_sitter_typescript::TAGS_QUERY,
    )
    .unwrap();
}
