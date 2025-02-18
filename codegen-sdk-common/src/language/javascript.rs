use super::Language;

lazy_static! {
    pub static ref Javascript: Language = Language::new(
        "javascript",
        "Javascript",
        tree_sitter_javascript::NODE_TYPES,
        &["js"],
        tree_sitter_javascript::LANGUAGE.into(),
        tree_sitter_javascript::TAGS_QUERY,
    )
    .unwrap();
}
