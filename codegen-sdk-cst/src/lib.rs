use std::path::PathBuf;

use codegen_sdk_common::{
    language::Language,
    traits::{CSTNode, FromNode},
    ParseError,
};
use codegen_sdk_macros::{include_language, parse_language};

pub trait CSTLanguage {
    type Program: CSTNode + FromNode + Send;
    fn language() -> &'static Language;
    fn parse(content: &str) -> Result<Self::Program, ParseError> {
        let tree = Self::language().parse_tree_sitter(content)?;
        Self::Program::from_node(tree.root_node())
    }
    fn parse_file(file_path: &PathBuf) -> Result<Self::Program, ParseError> {
        let content = std::fs::read_to_string(file_path)?;
        Self::parse(&content)
    }

    fn should_parse(file_path: &PathBuf) -> Result<bool, ParseError> {
        Ok(Self::language().file_extensions.contains(
            &file_path
                .extension()
                .ok_or(ParseError::Miscelaneous)?
                .to_str()
                .ok_or(ParseError::Miscelaneous)?,
        ))
    }
}
include_language!(python);
include_language!(typescript);
include_language!(tsx);
include_language!(jsx);
include_language!(javascript);
pub fn parse_file(file_path: &PathBuf) -> Result<Box<dyn CSTNode + Send>, ParseError> {
    parse_language!(python);
    parse_language!(typescript);
    parse_language!(tsx);
    parse_language!(jsx);
    parse_language!(javascript);
    Err(ParseError::UnknownLanguage)
}

#[cfg(test)]
mod tests {

    use codegen_sdk_common::traits::HasChildren;

    use super::*;
    #[test]
    fn test_snazzy_items() {
        let content = "
        class SnazzyItems {
            constructor() {
                this.items = [];
            }
        }
        ";
        let module = typescript::Typescript::parse(&content).unwrap();
        assert!(module.children().len() > 0);
    }
}
