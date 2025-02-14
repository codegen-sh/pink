pub fn format_cst(cst: &str) -> anyhow::Result<String> {
    let parsed = syn::parse_str::<syn::File>(cst)?;
    Ok(prettyplease::unparse(&parsed))
}
