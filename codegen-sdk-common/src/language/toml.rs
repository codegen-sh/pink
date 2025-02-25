use super::Language;
lazy_static! {
    pub static ref TOML: Language = Language::new(
        "toml",
        "TOML",
        tree_sitter_toml_ng::NODE_TYPES,
        &["toml"],
        tree_sitter_toml_ng::LANGUAGE.into(),
        "",
    )
    .unwrap();
}
