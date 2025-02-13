use std::{
    error::Error,
    fmt::{self, Display},
    path::PathBuf,
};

use codegen_sdk_common::{
    language::Language,
    traits::{CSTNode, FromNode},
};
use codegen_sdk_macros::{include_language, parse_language};

pub trait CSTLanguage {
    type Program: CSTNode + FromNode + Send;
    fn language() -> &'static Language;
    fn parse(content: &str) -> Result<Self::Program, Box<dyn Error>> {
        let tree = Self::language().parse_tree_sitter(content)?;
        Ok(Self::Program::from_node(tree.root_node()))
    }
    fn parse_file(file_path: &PathBuf) -> Result<Self::Program, Box<dyn Error>> {
        let content = std::fs::read_to_string(file_path)?;
        Self::parse(&content)
    }

    fn should_parse(file_path: &PathBuf) -> bool {
        Self::language()
            .file_extensions
            .contains(&file_path.extension().unwrap().to_str().unwrap())
    }
}
include_language!(python);
include_language!(typescript);
include_language!(tsx);
include_language!(jsx);
include_language!(javascript);
#[derive(Debug)]
struct ParseError {}
impl Error for ParseError {}
impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError")
    }
}
pub fn parse_file(file_path: &PathBuf) -> Result<Box<dyn CSTNode + Send>, Box<dyn Error>> {
    parse_language!(python);
    parse_language!(typescript);
    parse_language!(tsx);
    parse_language!(jsx);
    parse_language!(javascript);
    Err(Box::new(ParseError {}))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use codegen_sdk_common::traits::HasChildren;
    use tempfile::tempdir;

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
