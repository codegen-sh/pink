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
        tree_sitter_typescript::TSX_NODE_TYPES,
        &["ts", "tsx", "jsx", "js"],
        tree_sitter_typescript::LANGUAGE_TSX.into(),
        &QUERIES,
    )
    .unwrap();
}
