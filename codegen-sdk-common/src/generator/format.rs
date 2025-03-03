pub fn format_code(cst: &syn::File) -> anyhow::Result<String> {
    Ok(prettyplease::unparse(cst))
}
pub fn format_code_string(cst: &str) -> anyhow::Result<String> {
    let parsed = syn::parse_str::<syn::File>(cst)?;
    format_code(&parsed)
}
