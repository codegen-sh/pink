use std::collections::{BTreeMap, BTreeSet, VecDeque};

use codegen_sdk_common::{Language, naming::normalize_type_name, parser::TypeDefinition};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{node::Node, utils::get_from_node};
use crate::generator::{
    constants::TYPE_NAME,
    utils::{get_comment_type, get_from_for_enum},
};
#[derive(Debug)]
pub struct State<'a> {
    pub subenums: BTreeSet<String>,
    nodes: BTreeMap<String, Node<'a>>,
    language: &'a Language,
}
impl<'a> State<'a> {
    pub fn new(language: &'a Language) -> Self {
        let mut nodes = BTreeMap::new();
        let mut subenums = BTreeSet::new();
        let raw_nodes = language.nodes();
        for raw_node in raw_nodes {
            if raw_node.subtypes.is_empty() {
                let node = Node::new(raw_node, language);
                nodes.insert(node.normalize_name(), node);
            } else {
                subenums.insert(raw_node.type_name.clone());
            }
        }
        let mut ret = Self {
            nodes,
            subenums,
            language,
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
        log::info!("Adding field subenums");
        ret.add_field_subenums();
        ret
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
                let variants = self.get_variants(&node.type_name);
                self.add_subenum(name, &variants.iter().collect());
            }
        }
    }
    fn get_variants(&self, subenum: &str) -> Vec<TypeDefinition> {
        let comment = get_comment_type();
        let mut variants = vec![comment];
        for node in self.nodes.values() {
            log::debug!("Checking subenum: {} for {}", subenum, node.kind());
            if node.subenums.contains(&subenum.to_string()) {
                log::debug!("Found variant: {} for {}", node.kind(), subenum);
                variants.push(node.type_definition());
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
                        Ok(Self::#variant_name(#variant_name::from_node(node, buffer)?))
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
    pub fn get_enum(&self) -> TokenStream {
        let mut enum_tokens = Vec::new();
        let mut from_tokens = TokenStream::new();
        let mut subenums = Vec::new();
        for node in self.nodes.values() {
            enum_tokens.push(node.get_enum_tokens());
            let variant_name = node.normalize_name();
        }
        for subenum in self.subenums.iter() {
            assert!(
                self.get_variants(subenum).len() > 0,
                "Subenum {} has no variants",
                subenum
            );
            from_tokens.extend_one(self.get_from_node(subenum));
            subenums.push(format_ident!("{}", normalize_type_name(&subenum, true)));
        }
        let subenum_tokens = if !subenums.is_empty() {
            subenums.sort();
            subenums.dedup();
            quote! {
                #[subenum(#(#subenums(derive(Archive, Deserialize, Serialize))),*)]
            }
        } else {
            quote! {}
        };
        let enum_name = format_ident!("{}", TYPE_NAME);
        quote! {
            #subenum_tokens
        #[derive(Debug, Clone, Drive)]
        #[enum_delegate::implement(CSTNode)]
        pub enum #enum_name {
                #(#enum_tokens),*
            }
            #from_tokens
        }
    }
    pub fn get_structs(&self) -> TokenStream {
        let mut struct_tokens = TokenStream::new();
        for node in self.nodes.values() {
            struct_tokens.extend_one(node.get_struct_tokens());
        }
        struct_tokens
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use assert_tokenstreams_eq::assert_tokenstreams_eq;

    use super::*;
    use crate::generator::utils::get_serialize_bounds;

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
        let state = State::from(&nodes);
        let enum_tokens = state.get_enum();
        assert_tokenstreams_eq!(
            &quote! {
                #[derive(Debug, Clone)]
                pub enum NodeTypes {
                    Test(Test)
                }
                impl std::convert::From<Test> for NodeTypes {
                    fn from(variant: Test) -> Self {
                        Self::Test(variant)
                    }
                }
            },
            &enum_tokens
        );
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
        let state = State::from(&nodes);
        let enum_tokens = state.get_enum();
        assert_tokenstreams_eq!(
            &quote! {
                #[subenum(TestChildren(derive(Archive, Deserialize, Serialize)))]
                #[derive(Debug, Clone)]
                pub enum NodeTypes {
                    #[subenum(TestChildren)]
                    Child(Child),
                    #[subenum(TestChildren)]
                    ChildTwo(ChildTwo),
                    Test(Test)
                }
                impl std::convert::From<Child> for NodeTypes {
                    fn from(variant: Child) -> Self {
                        Self::Child(variant)
                    }
                }
                impl std::convert::From<Child> for TestChildren {
                    fn from(variant: Child) -> Self {
                        Self::Child(variant)
                    }
                }
                impl std::convert::From<ChildTwo> for NodeTypes {
                    fn from(variant: ChildTwo) -> Self {
                        Self::ChildTwo(variant)
                    }
                }
                impl std::convert::From<ChildTwo> for TestChildren {
                    fn from(variant: ChildTwo) -> Self {
                        Self::ChildTwo(variant)
                    }
                }
                impl std::convert::From<Test> for NodeTypes {
                    fn from(variant: Test) -> Self {
                        Self::Test(variant)
                    }
                }
                impl FromNode for TestChildren {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        match node.kind() {
                            "child" => Ok(Self::Child(node.from_node(node, buffer)?)),
                            "child_two" => Ok(Self::ChildTwo(node.from_node(node, buffer)?)),
                            _ => Err(ParseError::UnexpectedNode {
                                node_type: node.kind().to_string(),
                                backtrace: Backtrace::capture(),
                            }),                        }
                    }
                }
            },
            &enum_tokens
        );
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
        let state = State::from(&nodes);
        let enum_tokens = state.get_enum();
        assert_tokenstreams_eq!(
            &quote! {
                #[subenum(Definition(derive(Archive, Deserialize, Serialize)))]
                #[derive(Debug, Clone)]
                pub enum NodeTypes {
                    #[subenum(Definition)]
                    Class(Class),
                    #[subenum(Definition)]
                    Function(Function)
                }
                impl std::convert::From<Class> for NodeTypes {
                    fn from(variant: Class) -> Self {
                        Self::Class(variant)
                    }
                }
                impl std::convert::From<Class> for Definition {
                    fn from(variant: Class) -> Self {
                        Self::Class(variant)
                    }
                }
                impl std::convert::From<Function> for NodeTypes {
                    fn from(variant: Function) -> Self {
                        Self::Function(variant)
                    }
                }
                impl std::convert::From<Function> for Definition {
                    fn from(variant: Function) -> Self {
                        Self::Function(variant)
                    }
                }
                impl FromNode for Definition {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        match node.kind() {
                            "class" => Ok(Self::Class(node.from_node(node)?)),
                            "function" => Ok(Self::Function(node.from_node(node)?)),
                            _ => Err(ParseError::UnexpectedNode {
                                node_type: node.kind().to_string(),
                                backtrace: Backtrace::capture(),
                            }),                        }
                    }
                }
            },
            &enum_tokens
        );
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
        let state = State::from(&nodes);
        let enum_tokens = state.get_enum();
        assert_tokenstreams_eq!(
            &quote! {
                #[subenum(NodeCChildren(derive(Archive, Deserialize, Serialize)), NodeCField(derive(Archive, Deserialize, Serialize)))]
                #[derive(Debug, Clone)]
                pub enum NodeTypes {
                    #[subenum(NodeCChildren, NodeCField)]
                    NodeA(NodeA),
                    #[subenum(NodeCChildren, NodeCField)]
                    NodeB(NodeB),
                    NodeC(NodeC)
                }
                impl std::convert::From<NodeA> for NodeTypes {
                    fn from(variant: NodeA) -> Self {
                        Self::NodeA(variant)
                    }
                }
                impl std::convert::From<NodeA> for NodeCChildren {
                    fn from(variant: NodeA) -> Self {
                        Self::NodeA(variant)
                    }
                }
                impl std::convert::From<NodeA> for NodeCField {
                    fn from(variant: NodeA) -> Self {
                        Self::NodeA(variant)
                    }
                }

                impl std::convert::From<NodeB> for NodeTypes {
                    fn from(variant: NodeB) -> Self {
                        Self::NodeB(variant)
                    }
                }
                impl std::convert::From<NodeB> for NodeCChildren {
                    fn from(variant: NodeB) -> Self {
                        Self::NodeB(variant)
                    }
                }
                impl std::convert::From<NodeB> for NodeCField {
                    fn from(variant: NodeB) -> Self {
                        Self::NodeB(variant)
                    }
                }
                impl std::convert::From<NodeC> for NodeTypes {
                    fn from(variant: NodeC) -> Self {
                        Self::NodeC(variant)
                    }
                }
                impl FromNode for NodeCChildren {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        match node.kind() {
                            "node_a" => Ok(Self::NodeA(NodeA::from_node(node, buffer)?)),
                            "node_b" => Ok(Self::NodeB(NodeB::from_node(node, buffer)?)),
                            _ => Err(ParseError::UnexpectedNode {
                                node_type: node.kind().to_string(),
                                backtrace: Backtrace::capture(),
                            }),                        }
                    }
                }
                impl FromNode for NodeCField {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        match node.kind() {
                            "node_a" => Ok(Self::NodeA(NodeA::from_node(node, buffer)?)),
                            "node_b" => Ok(Self::NodeB(NodeB::from_node(node, buffer)?)),
                            _ => Err(ParseError::UnexpectedNode {
                                node_type: node.kind().to_string(),
                                backtrace: Backtrace::capture(),
                            }),                        }
                    }
                }

            },
            &enum_tokens
        );
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
        let state = State::from(&nodes);
        let struct_tokens = state.get_structs();
        let serialize_bounds = get_serialize_bounds();
        assert_tokenstreams_eq!(
            &quote! {
                #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
                #serialize_bounds
                pub struct Test {
                    start_byte: usize,
                    end_byte: usize,
                    _kind: std::string::String,
                    #[debug("[{},{}]", start_position.row, start_position.column)]
                    start_position: Point,
                    #[debug("[{},{}]", end_position.row, end_position.column)]
                    end_position: Point,
                    #[debug(ignore)]
                    buffer: Arc<Bytes>,
                    #[debug(ignore)]
                    kind_id: u16,
                }
                impl FromNode for Test {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        Ok(Self {
                            start_byte: node.start_byte(),
                            end_byte: node.end_byte(),
                            _kind: node.kind().to_string(),
                            start_position: node.start_position().into(),
                            end_position: node.end_position().into(),
                            buffer: buffer.clone(),
                            kind_id: node.kind_id(),
                        })
                    }
                }
                impl CSTNode for Test {
                    fn kind(&self) -> &str {
                        &self._kind
                    }
                    fn start_byte(&self) -> usize {
                        self.start_byte
                    }
                    fn end_byte(&self) -> usize {
                        self.end_byte
                    }
                    fn start_position(&self) -> Point {
                        self.start_position
                    }
                    fn end_position(&self) -> Point {
                        self.end_position
                    }
                    fn buffer(&self) -> &Bytes {
                        &self.buffer
                    }
                    fn kind_id(&self) -> u16 {
                        self.kind_id
                    }
                }
                impl HasChildren for Test {
                    type Child = Self;
                    fn children(&self) -> Vec<Self::Child> {
                        vec![]
                    }
                    fn children_by_field_name(&self, field_name: &str) -> Vec<Self::Child> {
                        match field_name {
                            _ => vec![],
                        }
                    }
                }

            },
            &struct_tokens
        );
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
        let state = State::from(&nodes);

        let variants = state.get_variants("parent");
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
        let mut state = State::from(&nodes);

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
