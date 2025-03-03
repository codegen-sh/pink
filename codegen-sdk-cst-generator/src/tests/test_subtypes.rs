use std::collections::HashMap;

use codegen_sdk_common::parser::{Fields, Node, TypeDefinition};

use crate::{Config, generate_cst, test_util::get_language};

#[test_log::test]
fn test_basic_subtypes() {
    // Define nodes with basic subtype relationships
    let nodes = vec![
        Node {
            type_name: "expression".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "binary_expression".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "unary_expression".to_string(),
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
            fields: None,
            children: None,
        },
        Node {
            type_name: "unary_expression".to_string(),
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

#[test_log::test]
fn test_subtypes_with_fields() {
    let nodes = vec![
        Node {
            type_name: "expression".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "binary_expression".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "literal".to_string(),
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
            type_name: "literal".to_string(),
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

#[test_log::test]
fn test_deeply_nested_subtypes() {
    let nodes = vec![
        // Top level statement type
        Node {
            type_name: "statement".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "declaration".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "expression_statement".to_string(),
                    named: true,
                },
            ],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        // Declaration with its subtypes
        Node {
            type_name: "declaration".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "function_declaration".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "class_declaration".to_string(),
                    named: true,
                },
            ],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        // Function declaration with its subtype
        Node {
            type_name: "function_declaration".to_string(),
            subtypes: vec![TypeDefinition {
                type_name: "method_declaration".to_string(),
                named: true,
            }],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        // Concrete node types
        Node {
            type_name: "method_declaration".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        Node {
            type_name: "class_declaration".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        },
        Node {
            type_name: "expression_statement".to_string(),
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
