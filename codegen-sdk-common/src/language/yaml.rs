use super::Language;
lazy_static! {
    pub static ref Yaml: Language = Language::new(
        "yaml",
        "Yaml",
        tree_sitter_yaml::NODE_TYPES,
        &["yaml", "yml"],
        tree_sitter_yaml::LANGUAGE.into(),
        "",
    )
    .unwrap();
}
