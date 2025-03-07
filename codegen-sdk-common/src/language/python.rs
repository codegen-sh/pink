use super::Language;
const PYTHON_TAGS_QUERY: &'static str = tree_sitter_python::TAGS_QUERY;
const EXTRA_TAGS_QUERY: &'static str = "
    (import_from_statement module_name: (dotted_name) @module name: (dotted_name) @name) @definition.import
    ";
lazy_static! {
    static ref TAGS_QUERY: String = [PYTHON_TAGS_QUERY, EXTRA_TAGS_QUERY].join("\n");
    pub static ref Python: Language = Language::new(
        "python",
        "Python",
        tree_sitter_python::NODE_TYPES,
        &["py"],
        tree_sitter_python::LANGUAGE.into(),
        &TAGS_QUERY,
    )
    .unwrap();
}
