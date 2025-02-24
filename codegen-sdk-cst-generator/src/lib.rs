#![feature(extend_one)]
mod generator;
#[double]
use codegen_sdk_common::language::Language;
pub use generator::generate_cst;
use mockall_double::double;
pub fn generate_cst_to_file(language: &Language) -> anyhow::Result<()> {
    let cst = generator::generate_cst(language)?;
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/{}.rs", out_dir, language.name());
    std::fs::write(out_file, cst)?;
    Ok(())
}
#[cfg(test)]
mod test_util {
    use codegen_sdk_common::{language::MockLanguage, parser::Node};
    pub fn get_language(nodes: Vec<Node>) -> MockLanguage {
        let mut language = get_language_no_nodes();
        language.expect_nodes().return_const(nodes);
        language
    }
    pub fn get_language_no_nodes() -> MockLanguage {
        let language = MockLanguage::default();
        language
    }
}
#[cfg(test)]
mod tests {
    mod test_subtypes;
    mod test_subtypes_children;
    mod test_subtypes_multiple_inheritance;
    mod test_subtypes_recursive;
}
