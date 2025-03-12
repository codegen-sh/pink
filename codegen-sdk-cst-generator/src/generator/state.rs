use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    sync::Arc,
};

#[double]
use codegen_sdk_common::language::Language;
use codegen_sdk_common::{naming::normalize_type_name, parser::TypeDefinition};
use mockall_double::double;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::node::Node;
use crate::{
    Config,
    generator::{
        constants::TYPE_NAME,
        utils::{get_comment_type, get_from_enum_to_ref, get_from_node, get_from_type},
    },
};
#[derive(Debug)]
pub struct State<'a> {
    pub subenums: BTreeSet<String>,
    config: Config,
    nodes: BTreeMap<String, Node<'a>>,
}
impl<'a> State<'a> {
    pub fn new(language: &'a Language, config: Config) -> Self {
        let mut nodes = BTreeMap::new();
        let mut subenums = BTreeSet::new();
        let raw_nodes = language.nodes();
        for raw_node in raw_nodes {
            if raw_node.subtypes.is_empty() {
                let node = Node::new(raw_node.clone(), language, config.clone());
                nodes.insert(node.normalize_name(), node);
            } else {
                subenums.insert(raw_node.type_name.clone());
            }
        }
        let mut ret = Self {
            nodes,
            subenums,
            config,
        };
        let mut subenums = VecDeque::new();
        for raw_node in raw_nodes.iter().filter(|n| !n.subtypes.is_empty()) {
            subenums.push_back(raw_node.clone());
        }
        while let Some(raw_node) = subenums.pop_front() {
            if raw_node
                .subtypes
                .iter()
                .any(|s| subenums.iter().any(|n| n.type_name == s.type_name))
            {
                subenums.push_back(raw_node);
            } else {
                // Add subtypes to the state
                ret.add_subenum(&raw_node.type_name, &raw_node.subtypes.iter().collect());
            }
        }

        log::info!("Adding child subenums");
        ret.add_child_subenums();
        log::info!("Adding missing fields");
        ret.add_missing_fields(language);
        log::info!("Adding field subenums");
        ret.add_field_subenums();
        ret
    }
    fn add_missing_fields(&mut self, language: &'a Language) {
        let mut to_insert = Vec::new();
        for node in self.nodes.values() {
            for field in &node.fields {
                for type_def in field.types() {
                    let name = type_def.normalize();
                    if !self.nodes.contains_key(&name)
                        && !self.subenums.contains(&type_def.type_name)
                    {
                        let node = codegen_sdk_common::parser::Node {
                            type_name: type_def.type_name.clone(),
                            subtypes: vec![],
                            named: type_def.named,
                            root: false,
                            fields: None,
                            children: None,
                        };
                        to_insert.push((name, node));
                    }
                }
            }
        }
        to_insert.dedup_by_key(|(name, _)| name.clone());
        for (name, node) in to_insert {
            self.nodes.insert(
                name,
                Node::new(Arc::new(node), language, self.config.clone()),
            );
        }
    }
    fn add_child_subenums(&mut self) {
        let keys = self.nodes.keys().cloned().collect::<Vec<_>>();
        for name in keys.into_iter() {
            let normalized_name = normalize_type_name(&name, true);
            let node = self.nodes.get(&normalized_name).unwrap();
            let mut children_types = node.get_children_names();
            if children_types.len() > 1 {
                children_types.sort();
                children_types.dedup();
                self.add_subenum(
                    &node.children_struct_name(),
                    &children_types.iter().collect(),
                );
            }
        }
    }
    fn add_field_subenums(&mut self) {
        let mut to_add: Vec<(String, Vec<TypeDefinition>)> = Vec::new();
        for node in self.nodes.values() {
            for field in &node.fields {
                log::debug!("Adding field subenum: {}", field.normalized_name());
                if field.types().len() > 1 {
                    to_add.push((
                        field.type_name(),
                        field.types().into_iter().cloned().collect(),
                    ));
                }
            }
        }
        for (name, types) in to_add.into_iter() {
            self.add_subenum(&name, &types.iter().collect());
        }
    }
    fn add_subenum(&mut self, name: &str, nodes: &Vec<&TypeDefinition>) {
        self.subenums.insert(name.to_string());
        let mut nodes = nodes.clone();
        let comment = get_comment_type();
        if self.nodes.contains_key(&comment.type_name) {
            nodes.push(&comment);
        }
        for node in nodes {
            let normalized_name = normalize_type_name(&node.type_name, node.named);
            if !self.subenums.contains(&node.type_name) {
                log::debug!("Adding subenum: {} to {}", name, normalized_name);
                if let Some(node) = self.nodes.get_mut(&normalized_name) {
                    node.add_subenum(name.to_string());
                }
            } else {
                let variants = self.get_variants(&node.type_name, true);
                self.add_subenum(name, &variants.iter().collect());
            }
        }
    }
    pub fn get_variants(&self, subenum: &str, include_comment: bool) -> Vec<TypeDefinition> {
        let mut variants = Vec::new();
        for node in self.nodes.values() {
            log::debug!("Checking subenum: {} for {}", subenum, node.kind());
            if node.subenums.contains(&subenum.to_string()) {
                log::debug!("Found variant: {} for {}", node.kind(), subenum);
                variants.push(node.type_definition());
            }
        }
        if include_comment {
            let comment = get_comment_type();
            if !variants.iter().any(|v| v.type_name == comment.type_name) {
                variants.push(comment);
            }
        }
        variants
    }
    fn get_variant_map(&self, enum_name: &str) -> BTreeMap<u16, TokenStream> {
        let mut variant_map = BTreeMap::new();
        for node in self.nodes.values() {
            let variant_name = format_ident!("{}", node.normalize_name());
            if node.subenums.contains(&enum_name.to_string()) {
                log::debug!("Adding variant: {} for {}", node.kind(), enum_name);
                variant_map.insert(
                    node.kind_id(),
                    quote! {
                        {
                            let (node, ids) = #variant_name::from_node(context, node)?;
                            Ok((Self::#variant_name(node), ids))
                        }
                    },
                );
            }
        }
        variant_map
    }
    // Implement the TSNode => CSTNode conversion trait on a given subenum
    fn get_from_node(&self, enum_name: &str) -> TokenStream {
        let variant_map = self.get_variant_map(enum_name);
        get_from_node(enum_name, true, &variant_map)
    }
    // Get the overarching enum for the nodes
    pub fn get_enum(&self, is_ref: bool) -> TokenStream {
        let mut enum_tokens = Vec::new();
        let mut from_tokens = TokenStream::new();
        let mut subenums = Vec::new();
        let mut subenum_name_map = HashMap::new();
        for name in self.subenums.iter() {
            subenum_name_map.insert(name.clone(), normalize_type_name(name, true));
        }
        let enum_name = if is_ref {
            format_ident!("{}Ref", TYPE_NAME)
        } else {
            format_ident!("{}", TYPE_NAME)
        };
        for node in self.nodes.values() {
            enum_tokens.push(node.get_enum_tokens(&subenum_name_map, is_ref));
            from_tokens.extend_one(get_from_type(&node.normalize_name(), &enum_name, is_ref));
        }
        if is_ref {
            let variants = self.get_node_struct_names();
            let ref_convert = get_from_enum_to_ref(&TYPE_NAME, &variants);
            from_tokens.extend_one(ref_convert);
        }
        for subenum in self.subenums.iter() {
            assert!(
                self.get_variants(subenum, true).len() > 0,
                "Subenum {} has no variants",
                subenum
            );
            if is_ref {
                let normalized_subenum = normalize_type_name(&subenum, true);
                let ref_name = format_ident!("{}Ref", normalized_subenum);
                subenums.push(ref_name);
                let variants = self
                    .get_variants(subenum, false)
                    .into_iter()
                    .map(|v| v.ident())
                    .collect();
                let ref_convert = get_from_enum_to_ref(&normalized_subenum, &variants);
                from_tokens.extend_one(ref_convert);
            } else {
                from_tokens.extend_one(self.get_from_node(subenum));
                subenums.push(format_ident!("{}", normalize_type_name(&subenum, true)));
            }
        }
        let subenum_tokens = if !subenums.is_empty() {
            subenums.sort();
            subenums.dedup();
            if self.config.serialize {
                quote! {
                    #[subenum(#(#subenums(derive(Archive, Deserialize, Serialize))),*)]
                }
            } else {
                quote! {
                    #[subenum(#(#subenums),*)]
                }
            }
        } else {
            quote! {}
        };
        let derive_header = if is_ref {
            quote! {
                #[derive(Debug, Eq, PartialEq, Hash, Clone, Ord, PartialOrd)]
            }
        } else {
            quote! {
            #[derive(Debug, Eq, PartialEq,Clone, Hash, salsa::Update, Delegate, Ord, PartialOrd)]
            #[delegate(
                CSTNode<'db1>
            )]
                }
        };
        quote! {
            #subenum_tokens
            #derive_header
        pub enum #enum_name<'db1> {
                #(#enum_tokens),*
            }
            #from_tokens
        }
    }
    pub fn get_structs(&self) -> TokenStream {
        let mut struct_tokens = TokenStream::new();
        let subenums = HashSet::from_iter(self.subenums.iter());
        for node in self.nodes.values() {
            struct_tokens.extend_one(node.get_struct_tokens(&subenums));
        }
        struct_tokens
    }
    pub fn get_node_for_struct_name(&self, name: &str) -> Option<&Node<'a>> {
        self.nodes.get(name)
    }
    pub fn get_node_for_raw_name(&self, name: &str) -> Option<&Node<'a>> {
        for node in self.nodes.values() {
            if node.kind() == name {
                return Some(node);
            }
        }
        None
    }
    pub fn get_subenum_variants(&self, name: &str, include_comment: bool) -> Vec<&Node<'a>> {
        let variants = self.get_variants(name, include_comment);
        let mut nodes = Vec::new();
        for variant in variants {
            if let Some(node) = self.get_node_for_struct_name(&variant.normalize()) {
                nodes.push(node);
            }
        }
        nodes
    }
    pub fn get_node_struct_names(&self) -> Vec<Ident> {
        self.nodes
            .values()
            .map(|node| format_ident!("{}", node.normalize_name()))
            .collect()
    }
    pub fn get_subenum_struct_names(&self) -> Vec<Ident> {
        self.subenums
            .iter()
            .map(|s| format_ident!("{}", normalize_type_name(s, true)))
            .collect()
    }
    pub fn nodes(&self) -> Vec<&Node<'a>> {
        self.nodes.values().collect()
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::test_util::{get_language, snapshot_tokens};

    #[test_log::test]
    fn test_get_enum() {
        let node = codegen_sdk_common::parser::Node {
            type_name: "test".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let nodes = vec![node];
        let language = get_language(nodes);
        let state = State::new(&language, Config::default());
        let enum_tokens = state.get_enum(false);
        insta::assert_debug_snapshot!(snapshot_tokens(&enum_tokens));
    }
    #[test_log::test]
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
        let language = get_language(nodes);
        let state = State::new(&language, Config::default());
        let enum_tokens = state.get_enum(false);
        insta::assert_debug_snapshot!(snapshot_tokens(&enum_tokens));
    }
    #[test_log::test]
    fn test_parse_children_subtypes() {
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
        let language = get_language(nodes);
        let state = State::new(&language, Config::default());
        let enum_tokens = state.get_enum(false);
        insta::assert_debug_snapshot!(snapshot_tokens(&enum_tokens));
    }
    #[test_log::test]
    fn test_add_field_subenums() {
        let node_a = codegen_sdk_common::parser::Node {
            type_name: "node_a".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let node_b = codegen_sdk_common::parser::Node {
            type_name: "node_b".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let field = codegen_sdk_common::parser::FieldDefinition {
            types: vec![
                TypeDefinition {
                    type_name: "node_a".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "node_b".to_string(),
                    named: true,
                },
            ],
            multiple: true,
            required: false,
        };
        let node_c = codegen_sdk_common::parser::Node {
            type_name: "node_c".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: Some(codegen_sdk_common::parser::Fields {
                fields: HashMap::from([("field".to_string(), field)]),
            }),
            children: None,
        };
        let nodes = vec![node_a, node_b, node_c];
        let language = get_language(nodes);
        let state = State::new(&language, Config::default());
        let enum_tokens = state.get_enum(false);
        insta::assert_debug_snapshot!(snapshot_tokens(&enum_tokens));
    }
    #[test_log::test]
    fn test_add_field_subenums_missing_node() {
        let node_a = codegen_sdk_common::parser::Node {
            type_name: "node_a".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: None,
        };
        let field = codegen_sdk_common::parser::FieldDefinition {
            types: vec![
                TypeDefinition {
                    type_name: "node_a".to_string(),
                    named: false,
                },
                TypeDefinition {
                    type_name: "node_b".to_string(),
                    named: true,
                },
            ],
            multiple: true,
            required: false,
        };
        let node_c = codegen_sdk_common::parser::Node {
            type_name: "node_c".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: Some(codegen_sdk_common::parser::Fields {
                fields: HashMap::from([("field".to_string(), field)]),
            }),
            children: None,
        };
        let nodes = vec![node_a, node_c];
        let language = get_language(nodes);
        let state = State::new(&language, Config::default());
        let enum_tokens = state.get_enum(false);
        let struct_tokens = state.get_structs();
        insta::assert_debug_snapshot!(snapshot_tokens(&enum_tokens));
        insta::assert_debug_snapshot!(snapshot_tokens(&struct_tokens));
    }
    #[test_log::test]
    fn test_get_structs() {
        let node = codegen_sdk_common::parser::Node {
            type_name: "test".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let nodes = vec![node];
        let language = get_language(nodes);
        let state = State::new(&language, Config::default());
        let struct_tokens = state.get_structs();
        insta::assert_debug_snapshot!(snapshot_tokens(&struct_tokens));
    }
    #[test_log::test]
    fn test_get_variants() {
        let node_a = codegen_sdk_common::parser::Node {
            type_name: "node_a".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let node_b = codegen_sdk_common::parser::Node {
            type_name: "node_b".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let parent = codegen_sdk_common::parser::Node {
            type_name: "parent".to_string(),
            subtypes: vec![
                TypeDefinition {
                    type_name: "node_a".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "node_b".to_string(),
                    named: true,
                },
            ],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let nodes = vec![node_a, node_b, parent];
        let language = get_language(nodes);
        let state = State::new(&language, Config::default());
        let variants = state.get_variants("parent", true);
        assert_eq!(
            vec![
                TypeDefinition {
                    type_name: "node_a".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "node_b".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "comment".to_string(),
                    named: true,
                },
            ],
            variants
        );
        let variants = state.get_variants("parent", false);
        assert_eq!(
            vec![
                TypeDefinition {
                    type_name: "node_a".to_string(),
                    named: true,
                },
                TypeDefinition {
                    type_name: "node_b".to_string(),
                    named: true,
                },
            ],
            variants
        );
    }
    #[test_log::test]
    fn test_add_subenum() {
        let node_a = codegen_sdk_common::parser::Node {
            type_name: "node_a".to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        };
        let nodes = vec![node_a];
        let language = get_language(nodes);
        let mut state = State::new(&language, Config::default());

        state.add_subenum(
            "TestEnum",
            &vec![&TypeDefinition {
                type_name: "node_a".to_string(),
                named: true,
            }],
        );
        assert!(state.subenums.contains("TestEnum"));

        let node = state.nodes.get("NodeA").unwrap();
        assert!(node.subenums.contains(&"TestEnum".to_string()));
    }
}
