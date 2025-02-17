use super::Language;

lazy_static! {
    pub static ref Typescript: Language = Language::new(
        "typescript",
        "Typescript",
        tree_sitter_typescript::TYPESCRIPT_NODE_TYPES,
        &["ts"],
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        tree_sitter_typescript::TAGS_QUERY,
    )
    .unwrap();
}
