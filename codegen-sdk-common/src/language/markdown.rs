use super::Language;
lazy_static! {
    pub static ref Markdown: Language = Language::new(
        "markdown",
        "Markdown",
        tree_sitter_md::NODE_TYPES_BLOCK,
        &["md"],
        tree_sitter_md::LANGUAGE.into(),
        "",
    )
    .unwrap();
}
