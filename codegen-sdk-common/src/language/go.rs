use super::Language;
lazy_static! {
    pub static ref Go: Language = Language::new(
        "go",
        "Go",
        tree_sitter_go::NODE_TYPES,
        &["go"],
        tree_sitter_go::LANGUAGE.into(),
        tree_sitter_go::TAGS_QUERY,
    )
    .unwrap();
}
