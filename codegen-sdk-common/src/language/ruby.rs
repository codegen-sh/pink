use super::Language;
lazy_static! {
    pub static ref Ruby: Language = Language::new(
        "ruby",
        "Ruby",
        tree_sitter_ruby::NODE_TYPES,
        &["rb"],
        tree_sitter_ruby::LANGUAGE.into(),
        tree_sitter_ruby::TAGS_QUERY,
    )
    .unwrap();
}
