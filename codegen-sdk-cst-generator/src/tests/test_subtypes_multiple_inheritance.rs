use assert_tokenstreams_eq::assert_tokenstreams_eq;
use codegen_sdk_common::parser::{Node, TypeDefinition};
use quote::quote;

use crate::{generate_cst, test_util::get_language};

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
    let output = generate_cst(&language).unwrap();
    let expected = quote! {
        use bytes::Bytes;
        use codegen_sdk_common::*;
        use derive_more::Debug;
        use rkyv::{Archive, Deserialize, Serialize};
        use subenum::subenum;
        use tree_sitter;

        #[derive(Debug, Clone)]
        #[subenum(
            ClassMember(derive(Archive, Deserialize, Serialize)),
            Declaration(derive(Archive, Deserialize, Serialize))
        )]
        pub enum NodeTypes {
            #[subenum(ClassMember, Declaration)]
            ClassMethod(ClassMethod),
        }

        impl std::convert::From<ClassMethod> for NodeTypes {
            fn from(variant: ClassMethod) -> Self {
                Self::ClassMethod(variant)
            }
        }

        impl std::convert::From<ClassMethod> for ClassMember {
            fn from(variant: ClassMethod) -> Self {
                Self::ClassMethod(variant)
            }
        }

        impl std::convert::From<ClassMethod> for Declaration {
            fn from(variant: ClassMethod) -> Self {
                Self::ClassMethod(variant)
            }
        }
    };
    assert_tokenstreams_eq!(&output, &expected);
}
