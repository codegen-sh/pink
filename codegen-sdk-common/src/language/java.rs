use super::Language;
lazy_static! {
    pub static ref Java: Language = Language {
        name: "java",
        struct_name: "Java",
        node_types: tree_sitter_java::NODE_TYPES,
        file_extensions: &["java"],
        tree_sitter_language: tree_sitter_java::LANGUAGE.into(),
        tag_query: tree_sitter_java::TAGS_QUERY,
    };
}
