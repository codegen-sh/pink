#![recursion_limit = "512"]
#![feature(trivial_bounds)]
use std::path::PathBuf;

use codegen_sdk_common::{ParseError, serialize::Cache, traits::CSTNode};
use codegen_sdk_macros::{include_languages, parse_languages};
use rkyv::{api::high::to_bytes_in, from_bytes};
mod language;
use language::CSTLanguage;
include_languages!();
pub fn parse_file(
    cache: &Cache,
    file_path: &PathBuf,
) -> Result<Box<dyn CSTNode + Send>, ParseError> {
    parse_languages!();
    Err(ParseError::UnknownLanguage)
}
pub mod query;

#[cfg(test)]
mod tests {

    use codegen_sdk_common::traits::HasChildren;

    use super::*;
    #[test_log::test]
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
