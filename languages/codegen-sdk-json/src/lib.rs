#![allow(unused)]
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
        let module = crate::cst::JSON::parse(&db, content.to_string())
            .as_ref()
            .unwrap();
        assert!(module.children().len() > 0);
    }
}
