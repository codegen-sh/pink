use std::{collections::HashMap};

use assert_tokenstreams_eq::assert_tokenstreams_eq;
use codegen_sdk_common::parser::{Children, Fields, Node, TypeDefinition};
use codegen_sdk_cst_generator::generate_cst;
use quote::quote;

#[test]
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

    let output = generate_cst(&nodes).unwrap();
    let expected = quote! {
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
            CallExpression(CallExpression),
        }

        impl std::convert::From<CallExpression> for NodeTypes {
            fn from(variant: CallExpression) -> Self {
                Self::CallExpression(variant)
            }
        }

        impl std::convert::From<CallExpression> for Expression {
            fn from(variant: CallExpression) -> Self {
                Self::CallExpression(variant)
            }
        }

        #[derive(Debug, Clone)]
        pub struct CallExpression {
            start_byte: usize,
            end_byte: usize,
            _kind: String,
            #[debug("[{},{}]", start_position.row, start_position.column)]
            start_position: Point,
            #[debug("[{},{}]", end_position.row, end_position.column)]
            end_position: Point,
            #[debug(ignore)]
            buffer: Arc<Bytes>,
            #[debug(ignore)]
            kind_id: u16,
            callee: Box<Expression>,
            children: Vec<Expression>,
        }

        impl HasChildren for CallExpression {
            type Child = Expression;
            fn children(&self) -> Vec<Self::Child> {
                let mut children: Vec<_> = self.children.iter().cloned().collect();
                children.push(self.callee.as_ref().clone());
                children
            }
            fn children_by_field_name(&self, field_name: &str) -> Vec<Self::Child> {
                match field_name {
                    "callee" => vec![self.callee.as_ref().clone()],
                    _ => vec![],
                }
            }
        }
    };
    assert_tokenstreams_eq!(&output, &expected);
}
