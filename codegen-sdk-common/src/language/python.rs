use super::Language;
lazy_static! {
    pub static ref Python: Language = Language::new(
        "python",
        "Python",
        tree_sitter_python::NODE_TYPES,
        &["py"],
        tree_sitter_python::LANGUAGE.into(),
        tree_sitter_python::TAGS_QUERY,
    )
    .unwrap();
}
