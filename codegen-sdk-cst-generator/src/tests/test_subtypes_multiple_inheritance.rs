use codegen_sdk_common::parser::{Node, TypeDefinition};

use crate::{Config, generate_cst, test_util::get_language};

#[test_log::test]
fn test_multiple_inheritance() {
    let nodes = vec![
        // Base types
        Node {
            type_name: "declaration".to_string(),
            subtypes: vec![TypeDefinition {
                type_name: "class_method".to_string(),
                named: true,
            }],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        Node {
            type_name: "class_member".to_string(),
            subtypes: vec![TypeDefinition {
                type_name: "class_method".to_string(),
                named: true,
            }],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        // ClassMethod inherits from both Declaration and ClassMember
        Node {
            type_name: "class_method".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
    ];

    let language = get_language(nodes);
    let output = generate_cst(&language, Config::default()).unwrap();
    insta::assert_debug_snapshot!(crate::test_util::snapshot_string(&output));
}
