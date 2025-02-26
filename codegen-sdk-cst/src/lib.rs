#![recursion_limit = "512"]
#![feature(trivial_bounds, extend_one)]
#![allow(unused)]

use std::path::PathBuf;

use codegen_sdk_common::{ParseError, serialize::Cache, traits::CSTNode};
use codegen_sdk_macros::{include_languages, parse_languages};
use rkyv::{api::high::to_bytes_in, from_bytes};
mod language;
pub use codegen_sdk_common::language::LANGUAGES;
pub use language::CSTLanguage;
include_languages!();
// pub fn parse_file<'db>(
//     db: &'db dyn salsa::Database,
//     cache: &'db Cache,
//     file_path: &'db PathBuf,
// ) -> Result<Box<dyn CSTNode<'db> + Send>, ParseError> {
//     // parse_languages!();
//     Err(ParseError::UnknownLanguage)
// }

#[cfg(test)]
mod tests {
    use codegen_sdk_common::traits::HasChildren;
    use derive_visitor::{Drive, Visitor};

    use super::*;
    use crate::typescript::ClassDeclaration;
    #[test_log::test]
    fn test_snazzy_items() {
        let content = "
        {
            \"name\": \"SnazzyItems\"
        }
        ";
        let module = json::JSON::parse(&content).unwrap();
        assert!(module.children().len() > 0);
    }
    #[derive(Visitor, Default)]
    #[visitor(ClassDeclaration(enter))]
    struct ClassVisitor {
        pub items: Vec<String>,
    }
    impl ClassVisitor {
        fn enter_class_declaration(&mut self, node: &typescript::ClassDeclaration) {
            self.items.push(node.name.source());
        }
    }
    #[test_log::test]
    fn test_visitor() {
        let content = "
        class SnazzyItems {
            constructor() {
                this.items = [];
            }
        }
        ";
        let module = typescript::Typescript::parse(&content).unwrap();
        let mut visitor = ClassVisitor::default();
        module.drive(&mut visitor);
        assert_eq!(visitor.items, vec!["SnazzyItems"]);
    }
}
