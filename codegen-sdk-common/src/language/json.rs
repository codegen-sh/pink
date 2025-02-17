use super::Language;
lazy_static! {
    pub static ref JSON: Language = Language::new(
        "json",
        "JSON",
        tree_sitter_json::NODE_TYPES,
        &["json"],
        tree_sitter_json::LANGUAGE.into(),
        "",
    )
    .unwrap();
}
