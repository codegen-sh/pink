use super::Language;
lazy_static! {
    pub static ref Query: Language = Language::new(
        "query",
        "Query",
        tree_sitter_query::NODE_TYPES,
        &[],
        tree_sitter_query::LANGUAGE.into(),
        "",
    )
    .unwrap();
}
