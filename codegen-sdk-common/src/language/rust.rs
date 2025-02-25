use super::Language;
lazy_static! {
    pub static ref Rust: Language = Language::new(
        "rust",
        "Rust",
        tree_sitter_rust::NODE_TYPES,
        &["rs"],
        tree_sitter_rust::LANGUAGE.into(),
        tree_sitter_rust::TAGS_QUERY,
    )
    .unwrap();
}
