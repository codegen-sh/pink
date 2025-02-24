use std::{path::PathBuf, sync::Arc};

use bytes::Bytes;
use codegen_sdk_common::{
    ParseError,
    language::Language,
    traits::{CSTNode, FromNode},
};
pub trait CSTLanguage {
    type Program: CSTNode + FromNode + Send;
    fn language() -> &'static Language;
    fn parse(content: &str) -> Result<Self::Program, ParseError> {
        let buffer = Bytes::from(content.as_bytes().to_vec());
        let tree = Self::language().parse_tree_sitter(content)?;
        if tree.root_node().has_error() {
            Err(ParseError::SyntaxError)
        } else {
            let buffer = Arc::new(buffer);
            Self::Program::from_node(tree.root_node(), &buffer)
        }
    }
    fn parse_file(file_path: &PathBuf) -> Result<Self::Program, ParseError> {
        let content = std::fs::read_to_string(file_path)?;
        let parsed = Self::parse(&content)?;
        Ok(parsed)
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
