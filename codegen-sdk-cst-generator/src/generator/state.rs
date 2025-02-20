use std::collections::{BTreeMap, HashMap};

use codegen_sdk_common::parser::TypeDefinition;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::node::Node;
use crate::generator::normalize_type_name;
#[derive(Default, Debug)]
pub struct State<'a> {
    pub enums: TokenStream,
    pub structs: TokenStream,
    subenums: Vec<String>,
    nodes: BTreeMap<String, Node<'a>>,
    pub variants: HashMap<String, Vec<TypeDefinition>>,
    pub anonymous_nodes: HashMap<String, String>,
}
impl<'a> From<&'a Vec<codegen_sdk_common::parser::Node>> for State<'a> {
    fn from(raw_nodes: &'a Vec<codegen_sdk_common::parser::Node>) -> Self {
        let mut nodes = BTreeMap::new();
        for raw_node in raw_nodes {
            if raw_node.subtypes.is_empty() {
                let node = Node::from(raw_node);
                nodes.insert(node.normalize_name(), node);
            }
        }
        let mut ret = State {
            nodes,
            ..Default::default()
        };
        for raw_node in raw_nodes {
            // Add subtypes to the state
            if !raw_node.subtypes.is_empty() {
                ret.add_subenum(
                    &raw_node.type_name,
                    &raw_node
                        .subtypes
                        .iter()
                        .map(|t| t.type_name.clone())
                        .collect(),
                );
            }
        }
        ret.add_child_subenums();
        ret
    }
}
impl<'a> State<'a> {
    fn add_child_subenums(&mut self) {
        let keys = self.nodes.keys().cloned().collect::<Vec<_>>();
        for name in keys.into_iter() {
            let normalized_name = normalize_type_name(&name);
            let node = self.nodes.get(&normalized_name).unwrap();
            let mut children_types = node.get_children_names();
            if children_types.len() > 1 {
                children_types.sort();
                children_types.dedup();
                self.add_subenum(&format!("{}Children", normalized_name), &children_types);
            }
        }
    }
    fn add_subenum(&mut self, name: &str, nodes: &Vec<String>) {
        self.subenums.push(name.to_string());
        for node in nodes {
            let normalized_name = normalize_type_name(node);
            log::info!("Adding subenum: {} to {}", name, normalized_name);
            let node = self.nodes.get_mut(&normalized_name).unwrap();
            node.add_subenum(name.to_string());
        }
    }
    // Get the overarching enum for the nodes
    pub fn get_enum(&self) -> TokenStream {
        let mut enum_tokens = Vec::new();
        let mut subenums = Vec::new();
        for node in self.nodes.values() {
            enum_tokens.push(node.get_enum_tokens());
            for subenum in node.subenums.clone() {
                subenums.push(format_ident!("{}", normalize_type_name(&subenum)));
            }
        }
        let subenum_tokens = if !subenums.is_empty() {
            subenums.sort();
            subenums.dedup();
            quote! {
                #[subenum(#(#subenums),*)]
            }
        } else {
            quote! {}
        };
        quote! {
            #[derive(Debug)]
            #subenum_tokens
            pub enum Types {
                #(#enum_tokens),*
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_enum() {
        let node = codegen_sdk_common::parser::Node {
            type_name: "test".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: None,
        };
        let nodes = vec![node];
        let state = State::from(&nodes);
        let enum_tokens = state.get_enum();
        assert_eq!(
            enum_tokens.to_string(),
            quote! {
                #[derive(Debug)]
                pub enum Types {
                    Test(Test)
                }
            }
            .to_string()
        );
    }
    #[test]
    fn test_parse_children() {
        let child = codegen_sdk_common::parser::Node {
            type_name: "child".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: None,
        };
        let child_two = codegen_sdk_common::parser::Node {
            type_name: "child_two".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: None,
        };
        let node = codegen_sdk_common::parser::Node {
            type_name: "test".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: Some(codegen_sdk_common::parser::Children {
                multiple: true,
                required: false,
                types: vec![
                    codegen_sdk_common::parser::TypeDefinition {
                        type_name: "child".to_string(),
                        named: true,
                    },
                    codegen_sdk_common::parser::TypeDefinition {
                        type_name: "child_two".to_string(),
                        named: true,
                    },
                ],
            }),
        };
        let nodes = vec![child, child_two, node];
        let state = State::from(&nodes);
        let enum_tokens = state.get_enum();
        assert_eq!(
            enum_tokens.to_string(),
            quote! {
                #[derive(Debug)]
                #[subenum(TestChildren)]
                pub enum Types {
                    #[subenum(TestChildren)]
                    Child(Child),
                    #[subenum(TestChildren)]
                    ChildTwo(ChildTwo),
                    Test(Test)
                }
            }
            .to_string()
        );
    }
    #[test]
    fn test_parse_children_subtypes() {
        env_logger::init();
        let definition = codegen_sdk_common::parser::Node {
            type_name: "definition".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "function".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "class".to_string(),
                    named: true,
                },
            ],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let class = codegen_sdk_common::parser::Node {
            type_name: "class".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: Some(codegen_sdk_common::parser::Children {
                multiple: true,
                required: false,
                types: vec![codegen_sdk_common::parser::TypeDefinition {
                    type_name: "definition".to_string(),
                    named: true,
                }],
            }),
        };
        let function = codegen_sdk_common::parser::Node {
            type_name: "function".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: None,
        };
        let nodes = vec![definition, class, function];
        let state = State::from(&nodes);
        let enum_tokens = state.get_enum();
        assert_eq!(
            enum_tokens.to_string(),
            quote! {
                #[derive(Debug)]
                #[subenum(Definition)]
                pub enum Types {
                    #[subenum(Definition)]
                    Class(Class),
                    #[subenum(Definition)]
                    Function(Function)
                }
            }
            .to_string()
        );
    }
}
