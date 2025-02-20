use codegen_sdk_common::naming::normalize_type_name;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::field::Field;
use crate::generator::utils::get_serialize_bounds;
#[derive(Debug)]
pub struct Node<'a> {
    raw: &'a codegen_sdk_common::parser::Node,
    pub subenums: Vec<String>,
    pub fields: Vec<Field<'a>>,
}
impl<'a> From<&'a codegen_sdk_common::parser::Node> for Node<'a> {
    fn from(raw: &'a codegen_sdk_common::parser::Node) -> Self {
        let mut fields = Vec::new();
        let normalized_name = normalize_type_name(&raw.type_name);
        if let Some(raw_fields) = &raw.fields {
            for (name, field) in raw_fields.fields.iter() {
                fields.push(Field::new(&normalized_name, name, field));
            }
        }
        fields.sort_by_key(|f| f.normalized_name().clone());
        Node {
            raw,
            subenums: Vec::new(),
            fields,
        }
    }
}
impl<'a> Node<'a> {
    pub fn normalize_name(&self) -> String {
        normalize_type_name(&self.raw.type_name)
    }
    pub fn add_subenum(&mut self, subenum: String) {
        if !self.subenums.contains(&subenum) {
            self.subenums.push(subenum);
        }
    }
    pub fn get_enum_tokens(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let subenum_names = &self
            .subenums
            .iter()
            .map(|s| format_ident!("{}", normalize_type_name(s)))
            .collect::<Vec<_>>();
        if subenum_names.is_empty() {
            quote! {
                #name(#name)
            }
        } else {
            quote! {
                #[subenum(#(#subenum_names), *)]
                #name(#name)
            }
        }
    }
    pub fn get_children_names(&self) -> Vec<String> {
        let mut children_names = vec![];
        if let Some(children) = &self.raw.children {
            children_names.extend(children.types.iter().map(|t| t.type_name.clone()));
        }
        for field in &self.fields {
            children_names.extend(field.types());
        }
        children_names
    }
    pub fn children_struct_name(&self) -> String {
        let children_names = self.get_children_names();
        match children_names.len() {
            0 => "()".to_string(),
            1 => normalize_type_name(&children_names[0]),
            _ => format!("{}Children", self.normalize_name()),
        }
    }
    fn has_children(&self) -> bool {
        self.raw.children.is_some()
    }
    fn get_children_field(&self) -> TokenStream {
        if self.has_children() {
            let children_type_name = format_ident!("{}", self.children_struct_name());
            quote! {
                #[rkyv(omit_bounds)]
                pub children: Vec<#children_type_name>,
            }
        } else {
            quote! {}
        }
    }
    pub fn get_struct_tokens(&self) -> TokenStream {
        let constructor = self.get_constructor();
        let struct_fields = self
            .fields
            .iter()
            .map(|f| f.get_struct_field())
            .collect::<Vec<_>>();
        let children_field = self.get_children_field();
        let name = format_ident!("{}", self.normalize_name());
        let serialize_bounds = get_serialize_bounds();
        quote! {
        #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
        #serialize_bounds
        pub struct #name {
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
            #children_field
            #(#struct_fields),*
        }
        #constructor
        }
    }
    fn get_children_constructor(&self) -> TokenStream {
        if self.has_children() {
            quote! {
                children: named_children_without_field_names(node, buffer)?
            }
        } else {
            quote! {}
        }
    }
    pub fn get_constructor(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let mut constructor_fields = Vec::new();
        for field in &self.fields {
            constructor_fields.push(field.get_constructor_field());
        }
        constructor_fields.push(self.get_children_constructor());

        quote! {
            impl FromNode for #name {
                fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                    Ok(Self {
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                        _kind: node.kind().to_string(),
                        start_position: node.start_position().into(),
                        end_position: node.end_position().into(),
                        buffer: buffer.clone(),
                        kind_id: node.kind_id(),
                        #(#constructor_fields),*
                    })
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use assert_tokenstreams_eq::assert_tokenstreams_eq;
    use codegen_sdk_common::parser::{FieldDefinition, Fields, TypeDefinition};

    use super::*;

    fn create_test_node(name: &str) -> codegen_sdk_common::parser::Node {
        codegen_sdk_common::parser::Node {
            type_name: name.to_string(),
            subtypes: vec![],
            named: false,
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
            named: false,
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
            named: false,
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
        let mut node = Node::from(&base_node);

        let tokens = node.get_enum_tokens();
        assert_eq!(tokens.to_string(), quote! { Test(Test) }.to_string());
        node.add_subenum("subenum".to_string());
        let tokens = node.get_enum_tokens();
        assert_eq!(
            tokens.to_string(),
            quote! {
                #[subenum(Subenum)]
                Test (Test)
            }
            .to_string()
        );
    }

    #[test]
    fn test_get_struct_tokens_simple() {
        let raw_node = create_test_node("test_node");
        let node = Node::from(&raw_node);
        let serialize_bounds = get_serialize_bounds();
        assert_tokenstreams_eq!(
            &node.get_struct_tokens(),
            &quote! {
                #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
                #serialize_bounds
                pub struct TestNode {
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

                impl FromNode for TestNode {
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
            }
        );
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
        let node = Node::from(&raw_node);
        let serialize_bounds = get_serialize_bounds();
        assert_tokenstreams_eq!(
            &node.get_struct_tokens(),
            &quote! {
                #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
                #serialize_bounds
                pub struct TestNode {
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
                    pub TestField: TestType
                }

                impl FromNode for TestNode {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        Ok(Self {
                            start_byte: node.start_byte(),
                            end_byte: node.end_byte(),
                            _kind: node.kind().to_string(),
                            start_position: node.start_position().into(),
                            end_position: node.end_position().into(),
                            buffer: buffer.clone(),
                            kind_id: node.kind_id(),
                            TestField: get_child_by_field_name(&node, "test_field", buffer)?,
                        })
                    }
                }
            }
        );
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
        let node = Node::from(&raw_node);
        let serialize_bounds = get_serialize_bounds();
        assert_tokenstreams_eq!(
            &node.get_struct_tokens(),
            &quote! {
                #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
                #serialize_bounds
                pub struct TestNode {
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
                    pub MultipleField: Vec<TestType>,
                    pub OptionalField: Option<TestType>,
                    pub RequiredField: TestType,
                }

                impl FromNode for TestNode {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        Ok(Self {
                            start_byte: node.start_byte(),
                            end_byte: node.end_byte(),
                            _kind: node.kind().to_string(),
                            start_position: node.start_position().into(),
                            end_position: node.end_position().into(),
                            buffer: buffer.clone(),
                            kind_id: node.kind_id(),
                            MultipleField: get_multiple_children_by_field_name(&node, "multiple_field", buffer)?,
                            OptionalField: get_optional_child_by_field_name(&node, "optional_field", buffer)?,
                            RequiredField: get_child_by_field_name(&node, "required_field", buffer)?,
                        })
                    }
                }
            }
        );
    }

    #[test]
    fn test_get_struct_tokens_with_children() {
        let raw_node =
            create_test_node_with_children("test_node", vec!["child_type_a", "child_type_b"]);
        let node = Node::from(&raw_node);
        let serialize_bounds = get_serialize_bounds();

        assert_tokenstreams_eq!(
            &node.get_struct_tokens(),
            &quote! {
                #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
                #serialize_bounds
                pub struct TestNode {
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
                    #[rkyv(omit_bounds)]
                    pub children: Vec<TestNodeChildren>,
                }

                impl FromNode for TestNode {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        Ok(Self {
                            start_byte: node.start_byte(),
                            end_byte: node.end_byte(),
                            _kind: node.kind().to_string(),
                            start_position: node.start_position().into(),
                            end_position: node.end_position().into(),
                            buffer: buffer.clone(),
                            kind_id: node.kind_id(),
                            children: named_children_without_field_names(node, buffer)?,
                        })
                    }
                }
            }
        );
    }

    #[test]
    fn test_get_struct_tokens_with_single_child_type() {
        let raw_node = create_test_node_with_children("test_node", vec!["child_type"]);
        let node = Node::from(&raw_node);
        let serialize_bounds = get_serialize_bounds();

        assert_tokenstreams_eq!(
            &node.get_struct_tokens(),
            &quote! {
                #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
                #serialize_bounds
                pub struct TestNode {
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
                    #[rkyv(omit_bounds)]
                    pub children: Vec<ChildType>,
                }

                impl FromNode for TestNode {
                    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                        Ok(Self {
                            start_byte: node.start_byte(),
                            end_byte: node.end_byte(),
                            _kind: node.kind().to_string(),
                            start_position: node.start_position().into(),
                            end_position: node.end_position().into(),
                            buffer: buffer.clone(),
                            kind_id: node.kind_id(),
                            children: named_children_without_field_names(node, buffer)?,
                        })
                    }
                }
            }
        );
    }
}
