use super::Language;
lazy_static! {
    pub static ref Python: Language = Language {
        name: "python",
        node_types: tree_sitter_python::NODE_TYPES,
        file_extensions: &["py"],
        tree_sitter_language: tree_sitter_python::LANGUAGE.into(),
    };
}
