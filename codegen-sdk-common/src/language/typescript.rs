use super::Language;
// const ADDITIONAL_QUERIES: &str = "
// (class_declaration
//   name: (type_identifier) @name) @definition.class
// ";
const ADDITIONAL_QUERIES: &str = "
";
lazy_static! {
    static ref QUERIES: String = format!(
        "{}{}",
        ADDITIONAL_QUERIES,
        tree_sitter_typescript::TAGS_QUERY
    );
    pub static ref Typescript: Language = Language::new(
        "typescript",
        "Typescript",
        tree_sitter_typescript::TYPESCRIPT_NODE_TYPES,
        &["ts"],
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        &QUERIES,
    )
    .unwrap();
}
