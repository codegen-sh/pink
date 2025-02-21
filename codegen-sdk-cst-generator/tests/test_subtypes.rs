use std::collections::HashMap;

use assert_tokenstreams_eq::assert_tokenstreams_eq;
use codegen_sdk_common::parser::{Children, Fields, Node, TypeDefinition};
use codegen_sdk_cst_generator::generate_cst;
use quote::quote;

#[test]
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

    let output = generate_cst(&nodes).unwrap();
    let expected = quote! {
        use std::{backtrace::Backtrace, sync::Arc};
        use bytes::Bytes;
        use codegen_sdk_common::*;
        use derive_more::Debug;
        use rkyv::{Archive, Deserialize, Serialize};
        use subenum::subenum;
        use tree_sitter;

        #[derive(Debug, Clone)]
        #[subenum(Expression(derive(Archive, Deserialize, Serialize)))]
        pub enum NodeTypes {
            #[subenum(Expression)]
            BinaryExpression(BinaryExpression),
            #[subenum(Expression)]
            UnaryExpression(UnaryExpression),
        }

        impl std::convert::From<BinaryExpression> for NodeTypes {
            fn from(variant: BinaryExpression) -> Self {
                Self::BinaryExpression(variant)
            }
        }

        impl std::convert::From<UnaryExpression> for NodeTypes {
            fn from(variant: UnaryExpression) -> Self {
                Self::UnaryExpression(variant)
            }
        }

        impl std::convert::From<BinaryExpression> for Expression {
            fn from(variant: BinaryExpression) -> Self {
                Self::BinaryExpression(variant)
            }
        }

        impl std::convert::From<UnaryExpression> for Expression {
            fn from(variant: UnaryExpression) -> Self {
                Self::UnaryExpression(variant)
            }
        }

        impl FromNode for Expression {
            fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                match node.kind() {
                    "binary_expression" => Ok(Self::BinaryExpression(BinaryExpression::from_node(node, buffer)?)),
                    "unary_expression" => Ok(Self::UnaryExpression(UnaryExpression::from_node(node, buffer)?)),
                    _ => Err(ParseError::UnexpectedNode {
                        node_type: node.kind().to_string(),
                        backtrace: Backtrace::capture(),
                    }),
                }
            }
        }
    };
    assert_tokenstreams_eq!(&output, &expected);
}

#[test]
fn test_nested_subtypes() {
    // Define nodes with nested subtype relationships
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
        // Declaration is both a statement subtype and has its own subtypes
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
        // Concrete node types
        Node {
            type_name: "function_declaration".to_string(),
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

    let output = generate_cst(&nodes).unwrap();
    let expected = quote! {
        #[subenum(Statement(derive(Archive, Deserialize, Serialize)), Declaration(derive(Archive, Deserialize, Serialize)))]
        #[derive(Debug, Clone)]
        pub enum Types {
            #[subenum(Statement, Declaration)]
            FunctionDeclaration(FunctionDeclaration),
            #[subenum(Statement, Declaration)]
            ClassDeclaration(ClassDeclaration),
            #[subenum(Statement)]
            ExpressionStatement(ExpressionStatement),
        }
        // ... expected impl blocks ...
    };
    assert_tokenstreams_eq!(&output, &expected);
}

#[test]
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

    let output = generate_cst(&nodes).unwrap();
    let expected = quote! {
        use std::{backtrace::Backtrace, sync::Arc};
        use bytes::Bytes;
        use codegen_sdk_common::*;
        use derive_more::Debug;
        use rkyv::{Archive, Deserialize, Serialize};
        use subenum::subenum;
        use tree_sitter;
        #[derive(Debug, Clone)]
        pub struct BinaryExpression {
            left: Box<Expression>,
            right: Box<Expression>,
            // ... other required fields ...
        }
        // ... expected impl blocks ...
    };
    assert_tokenstreams_eq!(&output, &expected);
}
