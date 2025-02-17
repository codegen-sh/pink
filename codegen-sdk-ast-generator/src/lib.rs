use codegen_sdk_common::language::Language;
mod generator;
pub fn generate_ast(language: &Language) -> anyhow::Result<()> {
    let ast = generator::generate_ast(language)?;
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/{}.rs", out_dir, language.name);
    std::fs::write(out_file, ast)?;
    Ok(())
}
