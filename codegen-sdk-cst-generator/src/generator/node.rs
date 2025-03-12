// Representation of a tree-sitter node without variants
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[double]
use codegen_sdk_common::language::Language;
use codegen_sdk_common::{naming::normalize_type_name, parser::TypeDefinition};
use mockall_double::double;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::field::Field;
use crate::{
    Config,
    generator::utils::{get_comment_type, get_serialize_bounds},
};

#[derive(Debug)]
pub struct Node<'a> {
    raw: Arc<codegen_sdk_common::parser::Node>,
    pub subenums: Vec<String>,
    pub fields: Vec<Field<'a>>,
    language: &'a Language,
    config: Config,
    normalized_name: String,
}
impl<'a> Node<'a> {
    pub fn new(
        raw: Arc<codegen_sdk_common::parser::Node>,
        language: &'a Language,
        config: Config,
    ) -> Self {
        let mut fields = Vec::new();
        let normalized_name = normalize_type_name(&raw.type_name, raw.named);
        if let Some(raw_fields) = &raw.fields {
            for (name, field) in raw_fields.fields.iter() {
                fields.push(Field::new(
                    &normalized_name,
                    name,
                    field.clone(),
                    language,
                    config.clone(),
                ));
            }
        }
        fields.sort_by_key(|f| f.normalized_name().clone());
        Node {
            raw,
            subenums: Vec::new(),
            fields,
            language,
            config: config,
            normalized_name,
        }
    }
    pub fn kind(&self) -> &str {
        &self.raw.type_name
    }
    pub fn kind_id(&self) -> u16 {
        self.language.kind_id(&self.raw.type_name, self.raw.named)
    }
    pub fn normalize_name(&self) -> String {
        self.normalized_name.clone()
    }
    pub fn type_definition(&self) -> TypeDefinition {
        TypeDefinition {
            type_name: self.raw.type_name.clone(),
            named: self.raw.named,
        }
    }
    pub fn add_subenum(&mut self, subenum: String) {
        if !self.subenums.contains(&subenum) {
            self.subenums.push(subenum);
        }
    }
    pub fn get_enum_tokens(
        &self,
        subenum_name_map: &HashMap<String, String>,
        is_ref: bool,
    ) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let value = if is_ref {
            quote! { &'db1 #name }
        } else {
            quote! { #name }
        };
        let subenum_names = &self
            .subenums
            .iter()
            .map(|s| subenum_name_map.get(s).unwrap_or(&s))
            .map(|s| {
                if is_ref {
                    format_ident!("{}Ref", s)
                } else {
                    format_ident!("{}", s)
                }
            })
            .collect::<Vec<_>>();
        if subenum_names.is_empty() {
            quote! {
                #name(#value<'db1>)
            }
        } else {
            quote! {
                #[subenum(#(#subenum_names), *)]
                #name(#value<'db1>)
            }
        }
    }
    pub fn get_children_names(&self) -> Vec<TypeDefinition> {
        let mut children_names = vec![];
        let comment = get_comment_type();
        if let Some(children) = &self.raw.children {
            children_names.extend(children.types.iter().cloned());
        }
        for field in &self.fields {
            children_names.extend(field.types().into_iter().cloned());
        }
        if children_names.len() > 0 && !children_names.contains(&comment) {
            children_names.push(comment);
        }
        children_names.sort();
        children_names.dedup();
        children_names
    }
    pub fn children_struct_name(&self) -> String {
        let children_names = self.get_children_names();
        match children_names.len() {
            0 => "Self".to_string(),
            1 => normalize_type_name(&children_names[0].type_name, children_names[0].named),
            _ => format!("{}Children", self.normalize_name()),
        }
    }
    pub fn has_children(&self) -> bool {
        self.raw.children.is_some()
    }
    fn get_children_field(&self) -> TokenStream {
        if self.has_children() {
            let bounds = if self.config.serialize {
                quote! {
                    #[rkyv(omit_bounds)]
                }
            } else {
                quote! {}
            };
            quote! {
                #bounds
                pub _children: Vec<indextree::NodeId>,
            }
        } else {
            quote! {}
        }
    }

    pub fn get_struct_tokens(&self, subenums: &HashSet<&String>) -> TokenStream {
        let constructor = self.get_constructor();
        let struct_fields = self
            .fields
            .iter()
            .map(|f| f.get_struct_field())
            .collect::<Vec<_>>();
        let children_field = self.get_children_field();
        let name = format_ident!("{}", self.normalize_name());
        let trait_impls = self.get_trait_implementations(subenums);
        let derives = if self.config.serialize {
            let serialize_bounds = get_serialize_bounds();
            quote! {
                #[derive(Debug, Deserialize, Clone, Archive, Serialize, Eq, PartialEq, salsa::Update)]
                #serialize_bounds
            }
        } else {
            quote! {
                #[derive(Debug, Eq, PartialEq, Clone, salsa::Update)]

            }
        };
        quote! {
            #derives
            pub struct #name<'db> {
                id: CSTNodeId<'db>,
                file_id: FileNodeId,
                start_byte: usize,
                end_byte: usize,
                start_position: Point<'db>,
                end_position: Point<'db>,
                #[debug(ignore)]
                buffer: Arc<Bytes>,
                is_error: bool,
                #children_field
                #(#struct_fields),*
            }
            #constructor
            #trait_impls
        }
    }
    fn get_children_constructor(&self) -> TokenStream {
        if self.has_children() {
            let children_type_name = format_ident!("{}", self.children_struct_name());
            quote! {
                let _children = named_children_without_field_names::<NodeTypes<'db>, #children_type_name<'db>>(context, node)?;
                for child in _children.iter().cloned() {
                    ids.push(child);
                }
            }
        } else {
            quote! {}
        }
    }
    pub fn get_constructor(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let mut constructor_fields = Vec::new();
        let mut constructor_names = Vec::new();
        for field in &self.fields {
            constructor_fields.push(field.get_constructor_field());
            constructor_names.push(format_ident!("{}", field.name()));
        }
        constructor_fields.push(self.get_children_constructor());
        if self.has_children() {
            constructor_names.push(format_ident!("_children"));
        }

        quote! {
            impl<'db> FromNode<'db, NodeTypes<'db>> for #name<'db> {
                fn from_node(context: &mut ParseContext<'db, NodeTypes<'db>>, node: tree_sitter::Node) -> Result<(Self, Vec<indextree::NodeId>), ParseError> {
                    let start_position = Point::from(context.db, node.start_position());
                    let end_position = Point::from(context.db, node.end_position());
                    let id = CSTNodeId::new(context.db, context.file_id, node.id(), context.root);
                    let mut ids = Vec::new();
                    #(#constructor_fields)*
                    Ok((Self {
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                        start_position: start_position,
                        end_position: end_position,
                        buffer: context.buffer.clone(),
                        is_error: node.is_error(),
                        id,
                        file_id: context.file_id.clone(),
                        #(#constructor_names),*
                }, ids))
                }
            }
        }
    }
    fn get_children_impl(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());

        let children_type_name = self.children_struct_name();
        let children_type_generic;
        if children_type_name != "Self" {
            let children_type_ident = format_ident!("{}Ref", children_type_name);
            children_type_generic = quote! {#children_type_ident<'db2>};
        } else {
            children_type_generic = quote! {#name<'db2>};
        }

        let children_field = self.get_children_field_impl();
        let children_by_field_name = self.get_children_by_field_name_impl();
        let children_by_field_id = self.get_children_by_field_id_impl();
        quote! {
            impl<'db1> HasChildren<'db1, NodeTypes<'db1>> for #name<'db1> {
                type Child<'db2> = #children_type_generic where Self: 'db2;
                #children_field
                #children_by_field_name
                #children_by_field_id
            }
        }
    }
    pub fn get_trait_implementations(&self, subenums: &HashSet<&String>) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let children_impl = self.get_children_impl();
        let getters = self
            .fields
            .iter()
            .map(|f| f.get_field_getter(subenums))
            .collect::<Vec<_>>();
        let kind_name = &self.raw.type_name;
        let is_named = self.raw.named;
        let kind_id = self.kind_id();
        quote! {
            impl<'db> #name<'db> {
                const KIND_NAME: &'static str = #kind_name;
                #(#getters)*
            }
            impl<'db> CSTNode<'db> for #name<'db> {
                fn kind_name(&self) -> &str {
                    &Self::KIND_NAME
                }
                fn start_byte(&self) -> usize {
                    self.start_byte
                }
                fn end_byte(&self) -> usize {
                    self.end_byte
                }
                fn start_position(&self) -> Point<'db> {
                    self.start_position
                }
                fn end_position(&self) -> Point<'db> {
                    self.end_position
                }
                fn buffer(&self) -> &Bytes {
                    &self.buffer
                }
                fn kind_id(&self) -> u16 {
                    #kind_id
                }
                fn is_error(&self) -> bool {
                    self.is_error
                }
                fn is_named(&self) -> bool {
                    #is_named
                }
                fn id(&self) -> CSTNodeId<'db> {
                    self.id
                }
                fn file_id(&self) -> FileNodeId {
                    self.file_id
                }
            }
            #children_impl
            impl<'db> std::hash::Hash for #name<'db> {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    self.id.hash(state);
                }
            }
            impl<'db> PartialOrd for #name<'db> {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }
            impl<'db> Ord for #name<'db> {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    let res = self.start_byte().cmp(&other.start_byte());
                    if res == std::cmp::Ordering::Equal {
                        self.end_byte().cmp(&other.end_byte())
                    } else {
                        res
                    }
                }
            }

        }
    }
    fn get_children_field_impl(&self) -> TokenStream {
        let mut children_fields = Vec::new();
        let num_children = self.get_children_names().len();
        if num_children == 0 {
            return quote! {
                fn children<'db2>(&'db2 self, context: &'db2 Tree< NodeTypes<'db2>>) -> Vec<Self::Child<'db2>> {
                    vec![]
                }
            };
        }
        let convert_children = num_children > 1;
        for field in &self.fields {
            children_fields.push(field.get_children_field(convert_children));
        }

        let children_init = if self.has_children() {
            quote! {
                self._children.iter().map(|c| context.get(c).unwrap().as_ref().try_into().unwrap()).collect()
            }
        } else {
            quote! {
                vec![]
            }
        };
        quote! {
            fn children<'db2>(&'db2 self, context: &'db2 Tree< NodeTypes<'db2>>) -> Vec<Self::Child<'db2>> {
                let mut children: Vec<Self::Child<'db2>> = #children_init;
                #(#children_fields;)*
                children.sort_by_key(|c| c.start_byte());
                children
            }
        }
    }
    fn get_children_by_field_name_impl(&self) -> TokenStream {
        let convert_children = self.get_children_names().len() > 1;
        let field_matches = self
            .fields
            .iter()
            .map(|f| f.get_children_by_field_name_field(convert_children))
            .collect::<Vec<_>>();

        quote! {
            fn children_by_field_name<'db2>(&'db2 self, context: &'db2 Tree< NodeTypes<'db2>>, field_name: &str) -> Vec<Self::Child<'db2>> {
                match field_name {
                    #(#field_matches,)*
                    _ => vec![],
                }
            }
        }
    }
    fn get_children_by_field_id_impl(&self) -> TokenStream {
        let convert_children = self.get_children_names().len() > 1;
        let field_matches = self
            .fields
            .iter()
            .map(|f| f.get_children_by_field_id_field(convert_children))
            .collect::<Vec<_>>();

        quote! {
            fn children_by_field_id<'db2>(&'db2 self, context: &'db2 Tree< NodeTypes<'db2>>, field_id: u16) -> Vec<Self::Child<'db2>> {
                match field_id {
                    #(#field_matches,)*
                    _ => vec![],
                }
            }
        }
    }
    pub fn get_field_for_field_name(&self, field_name: &str) -> Option<&Field<'a>> {
        self.fields.iter().find(|f| f.name() == field_name)
    }
}
#[cfg(test)]
mod tests {
    use codegen_sdk_common::parser::{FieldDefinition, Fields, TypeDefinition};

    use super::*;
    use crate::test_util::{get_language, get_language_no_nodes, snapshot_tokens};

    fn create_test_node(name: &str) -> codegen_sdk_common::parser::Node {
        codegen_sdk_common::parser::Node {
            type_name: name.to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: None,
        }
    }

    fn create_test_node_with_fields(
        name: &str,
        fields: Vec<(&str, FieldDefinition)>,
    ) -> codegen_sdk_common::parser::Node {
        codegen_sdk_common::parser::Node {
            type_name: name.to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: Some(Fields {
                fields: fields
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v))
                    .collect(),
            }),
            children: None,
        }
    }

    fn create_test_node_with_children(
        name: &str,
        child_types: Vec<&str>,
    ) -> codegen_sdk_common::parser::Node {
        codegen_sdk_common::parser::Node {
            type_name: name.to_string(),
            subtypes: vec![],
            named: true,
            root: false,
            fields: None,
            children: Some(codegen_sdk_common::parser::Children {
                types: child_types
                    .into_iter()
                    .map(|t| TypeDefinition {
                        type_name: t.to_string(),
                        named: true,
                    })
                    .collect(),
                multiple: false,
                required: true,
            }),
        }
    }

    #[test_log::test]
    fn test_get_enum_tokens() {
        let base_node = create_test_node("test");
        let language = get_language_no_nodes();
        let mut node = Node::new(Arc::new(base_node), &language, Config::default());
        let mut subenum_name_map = HashMap::new();
        for subenum in &node.subenums {
            subenum_name_map.insert(subenum.clone(), normalize_type_name(subenum, true));
        }
        let tokens = node.get_enum_tokens(&subenum_name_map, false);
        insta::assert_debug_snapshot!(snapshot_tokens(&tokens));

        node.add_subenum("subenum".to_string());
        let tokens = node.get_enum_tokens(&subenum_name_map, false);
        insta::assert_debug_snapshot!(snapshot_tokens(&tokens));
    }

    #[test]
    fn test_get_struct_tokens_simple() {
        let raw_node = create_test_node("test_node");
        let language = get_language_no_nodes();
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens(&HashSet::new())));
    }

    #[test]
    fn test_get_struct_tokens_with_fields() {
        let raw_node = create_test_node_with_fields(
            "test_node",
            vec![(
                "test_field",
                FieldDefinition {
                    types: vec![TypeDefinition {
                        type_name: "test_type".to_string(),
                        named: true,
                    }],
                    multiple: false,
                    required: true,
                },
            )],
        );
        let language = get_language_no_nodes();
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens(&HashSet::new())));
    }

    #[test]
    fn test_get_struct_tokens_complex() {
        let raw_node = create_test_node_with_fields(
            "test_node",
            vec![
                (
                    "required_field",
                    FieldDefinition {
                        types: vec![TypeDefinition {
                            type_name: "test_type".to_string(),
                            named: true,
                        }],
                        multiple: false,
                        required: true,
                    },
                ),
                (
                    "optional_field",
                    FieldDefinition {
                        types: vec![TypeDefinition {
                            type_name: "test_type".to_string(),
                            named: true,
                        }],
                        multiple: false,
                        required: false,
                    },
                ),
                (
                    "multiple_field",
                    FieldDefinition {
                        types: vec![TypeDefinition {
                            type_name: "test_type".to_string(),
                            named: true,
                        }],
                        multiple: true,
                        required: true,
                    },
                ),
            ],
        );
        let nodes = vec![raw_node.clone()];
        let language = get_language(nodes);
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens(&HashSet::new())));
    }

    #[test_log::test]
    fn test_get_struct_tokens_with_children() {
        let raw_node =
            create_test_node_with_children("test_node", vec!["child_type_a", "child_type_b"]);
        let language = get_language_no_nodes();
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens(&HashSet::new())));
    }

    #[test]
    fn test_get_struct_tokens_with_single_child_type() {
        let raw_node = create_test_node_with_children("test_node", vec!["child_type"]);
        let language = get_language_no_nodes();
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens(&HashSet::new())));
    }

    #[test]
    fn test_get_trait_implementations() {
        let raw_node = create_test_node("test_node");
        let language = get_language_no_nodes();
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(
            &node.get_trait_implementations(&HashSet::new())
        ));
    }

    #[test]
    fn test_get_children_field_impl() {
        let raw_node = create_test_node_with_fields(
            "test_node",
            vec![(
                "test_field",
                FieldDefinition {
                    types: vec![TypeDefinition {
                        type_name: "test_type".to_string(),
                        named: true,
                    }],
                    multiple: false,
                    required: true,
                },
            )],
        );
        let language = get_language_no_nodes();
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_children_field_impl()));
    }

    #[test]
    fn test_get_children_by_field_name_impl() {
        let raw_node = create_test_node_with_fields(
            "test_node",
            vec![(
                "test_field",
                FieldDefinition {
                    types: vec![TypeDefinition {
                        type_name: "test_type".to_string(),
                        named: true,
                    }],
                    multiple: false,
                    required: true,
                },
            )],
        );
        let language = get_language_no_nodes();
        let node = Node::new(Arc::new(raw_node), &language, Config::default());
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_children_by_field_name_impl()));
    }
}
