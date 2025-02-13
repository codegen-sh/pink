mod generator;
pub(crate) mod parser;
use codegen_sdk_common::language::Language;
use std::error::Error;

pub fn generate_cst(language: &Language) -> Result<(), Box<dyn Error>> {
    let node_types = parser::parse_node_types(language)?;
    let cst = generator::generate_cst(&node_types)?;
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_file = format!("{}/{}.rs", out_dir, language.name);
    std::fs::write(out_file, cst)?;
    Ok(())
}
