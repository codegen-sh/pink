use codegen_sdk_common::parser::{Children, Node, TypeDefinition};

use crate::{Config, generate_cst, test_util::get_language};

#[test_log::test]
fn test_subtypes_with_children() {
    let nodes = vec![
        // A block can contain multiple statements
        Node {
            type_name: "block".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: Some(Children {
                multiple: true,
                required: false,
                types: vec![TypeDefinition {
                    type_name: "statement".to_string(),
                    named: true,
                }],
            }),
        },
        // Statement is a subtype with its own subtypes
        Node {
            type_name: "statement".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "if_statement".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "return_statement".to_string(),
                    named: true,
                },
            ],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        // Concrete statement types
        Node {
            type_name: "if_statement".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: Some(Children {
                multiple: false,
                required: true,
                types: vec![TypeDefinition {
                    type_name: "block".to_string(),
                    named: true,
                }],
            }),
        },
        Node {
            type_name: "return_statement".to_string(),
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
