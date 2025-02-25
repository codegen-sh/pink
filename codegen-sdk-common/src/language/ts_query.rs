use super::Language;
lazy_static! {
    pub static ref Query: Language = Language::new(
        "ts_query",
        "Query",
        tree_sitter_query::NODE_TYPES,
        &["scm"],
        tree_sitter_query::LANGUAGE.into(),
        "",
    )
    .unwrap();
}
