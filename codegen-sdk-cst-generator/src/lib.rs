mod generator;
use codegen_sdk_common::language::Language;
pub fn generate_cst(language: &Language) -> anyhow::Result<()> {
    let node_types = language.nodes();
    let cst = generator::generate_cst(&node_types)?;
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/{}.rs", out_dir, language.name);
    std::fs::write(out_file, cst)?;
    Ok(())
}
