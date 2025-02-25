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
    use std::{fmt::Debug, num::NonZeroU16};

    use codegen_sdk_common::{language::MockLanguage, parser::Node};
    use proc_macro2::TokenStream;

    use super::generator::format::format_cst;
    // Removes quotes from the string when using insta::assert_debug_snapshot!
    pub struct StringDebug {
        pub string: String,
    }
    impl Debug for StringDebug {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.string)
        }
    }
    pub fn get_language(nodes: Vec<Node>) -> MockLanguage {
        let mut language = get_language_no_nodes();
        for (idx, node) in nodes.clone().into_iter().enumerate() {
            language
                .expect_kind_id()
                .withf(move |name: &str, named: &bool| {
                    name == &node.type_name && named == &node.named
                })
                .return_const(idx as u16);
        }
        language.expect_nodes().return_const(nodes);
        language
    }
    pub fn get_language_no_nodes() -> MockLanguage {
        let mut language = MockLanguage::default();
        language.expect_kind_id().return_const(0 as u16);
        language
            .expect_field_id()
            .return_const(Some(NonZeroU16::new(1).unwrap()));
        language
    }
    pub fn snapshot_string(string: &str) -> StringDebug {
        let formatted = format_cst(string).unwrap_or_else(|_| string.to_string());
        StringDebug { string: formatted }
    }
    pub fn snapshot_tokens(tokens: &TokenStream) -> StringDebug {
        let string = tokens.to_string();
        snapshot_string(&string)
    }
}
#[cfg(test)]
mod tests {
    mod test_subtypes;
    mod test_subtypes_children;
    mod test_subtypes_multiple_inheritance;
    mod test_subtypes_recursive;
}
