use super::Language;
lazy_static! {
    pub static ref Java: Language = Language::new(
        "java",
        "Java",
        tree_sitter_java::NODE_TYPES,
        &["java"],
        tree_sitter_java::LANGUAGE.into(),
        tree_sitter_java::TAGS_QUERY,
    )
    .unwrap();
}
