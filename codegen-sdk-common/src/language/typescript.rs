use super::Language;

lazy_static! {
    pub static ref Typescript: Language = Language {
        name: "typescript",
        struct_name: "Typescript",
        node_types: tree_sitter_typescript::TYPESCRIPT_NODE_TYPES,
        file_extensions: &["ts"],
        tree_sitter_language: tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        tag_query: tree_sitter_typescript::TAGS_QUERY,
    };
}
