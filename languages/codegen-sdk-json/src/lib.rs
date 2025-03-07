#![allow(unused, irrefutable_let_patterns)]
pub mod cst {
    include!(concat!(env!("OUT_DIR"), "/json.rs"));
}
pub mod ast {
    include!(concat!(env!("OUT_DIR"), "/json-ast.rs"));
}
#[cfg(test)]
mod tests {
    use codegen_sdk_common::traits::HasChildren;
    use codegen_sdk_cst::CSTLanguage;

    use super::*;
    #[test_log::test]
    fn test_snazzy_items() {
        let content = "
        {
            \"name\": \"SnazzyItems\"
        }
        ";
        let db = codegen_sdk_cst::CSTDatabase::default();
        let module = crate::cst::JSON::parse(&db, content.to_string()).unwrap();
        let (root, tree, _) = module;
        assert!(root.children(tree).len() > 0);
    }
}
