use std::collections::HashMap;

use codegen_sdk_common::parser::{Children, Fields, Node, TypeDefinition};

use crate::{Config, generate_cst, test_util::get_language};
#[test_log::test]
fn test_recursive_subtypes() {
    let nodes = vec![
        // Expression can contain other expressions recursively
        Node {
            type_name: "expression".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "binary_expression".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "call_expression".to_string(),
                    named: true,
                },
            ],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        Node {
            type_name: "binary_expression".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: Some(Fields {
                fields: HashMap::from([
                    (
                        "left".to_string(),
                        codegen_sdk_common::parser::FieldDefinition {
                            types: vec![TypeDefinition {
                                type_name: "expression".to_string(),
                                named: true,
                            }],
                            multiple: false,
                            required: true,
                        },
                    ),
                    (
                        "right".to_string(),
                        codegen_sdk_common::parser::FieldDefinition {
                            types: vec![TypeDefinition {
                                type_name: "expression".to_string(),
                                named: true,
                            }],
                            multiple: false,
                            required: true,
                        },
                    ),
                ]),
            }),
            children: None,
        },
        Node {
            type_name: "call_expression".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: Some(Fields {
                fields: HashMap::from([(
                    "callee".to_string(),
                    codegen_sdk_common::parser::FieldDefinition {
                        types: vec![TypeDefinition {
                            type_name: "expression".to_string(),
                            named: true,
                        }],
                        multiple: false,
                        required: true,
                    },
                )]),
            }),
            children: Some(Children {
                multiple: true,
                required: false,
                types: vec![TypeDefinition {
                    type_name: "expression".to_string(),
                    named: true,
                }],
            }),
        },
    ];

    let language = get_language(nodes);
    let output = generate_cst(&language, Config::default()).unwrap();
    insta::assert_debug_snapshot!(crate::test_util::snapshot_string(&output));
}
