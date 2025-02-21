use std::sync::Arc;

use assert_tokenstreams_eq::assert_tokenstreams_eq;
use codegen_sdk_common::parser::{Children, Node, TypeDefinition};
use codegen_sdk_cst_generator::generate_cst;
use quote::quote;

#[test]
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

    let output = generate_cst(&nodes).unwrap();
    let expected = quote! {
        use bytes::Bytes;
        use codegen_sdk_common::*;
        use derive_more::Debug;
        use rkyv::{Archive, Deserialize, Serialize};
        use subenum::subenum;
        use tree_sitter;

        #[derive(Debug, Clone)]
        pub struct Block {
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
            children: Vec<Statement>,
        }

        impl HasChildren for Block {
            type Child = Statement;
            fn children(&self) -> Vec<Self::Child> {
                self.children.iter().cloned().collect()
            }
            fn children_by_field_name(&self, field_name: &str) -> Vec<Self::Child> {
                match field_name {
                    _ => vec![],
                }
            }
        }
    };
    assert_tokenstreams_eq!(&expected, &output);
}
