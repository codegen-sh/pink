#![recursion_limit = "2048"]
#![allow(non_snake_case)]
#![allow(unused)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/typescript.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/typescript-ast.rs"));
}

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
