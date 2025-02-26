#[double]
use codegen_sdk_common::language::Language;
use codegen_sdk_common::{naming::normalize_type_name, parser::TypeDefinition};
use mockall_double::double;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::field::Field;
use crate::generator::utils::{get_comment_type, get_serialize_bounds};
#[derive(Debug)]
pub struct Node<'a> {
    raw: &'a codegen_sdk_common::parser::Node,
    pub subenums: Vec<String>,
    pub fields: Vec<Field<'a>>,
    language: &'a Language,
}
impl<'a> Node<'a> {
    pub fn new(raw: &'a codegen_sdk_common::parser::Node, language: &'a Language) -> Self {
        let mut fields = Vec::new();
        let normalized_name = normalize_type_name(&raw.type_name, raw.named);
        if let Some(raw_fields) = &raw.fields {
            for (name, field) in raw_fields.fields.iter() {
                fields.push(Field::new(&normalized_name, name, field, language));
            }
        }
        fields.sort_by_key(|f| f.normalized_name().clone());
        Node {
            raw,
            subenums: Vec::new(),
            fields,
            language,
        }
    }
    pub fn kind(&self) -> &str {
        &self.raw.type_name
    }
    pub fn kind_id(&self) -> u16 {
        self.language.kind_id(&self.raw.type_name, self.raw.named)
    }
    pub fn normalize_name(&self) -> String {
        normalize_type_name(&self.raw.type_name, self.raw.named)
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
    pub fn get_enum_tokens(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let subenum_names = &self
            .subenums
            .iter()
            .map(|s| format_ident!("{}", normalize_type_name(s, true)))
            .collect::<Vec<_>>();
        if subenum_names.is_empty() {
            quote! {
                #name(#name<'db>)
            }
        } else {
            quote! {
                #[subenum(#(#subenum_names), *)]
                #name(#name<'db>)
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
    fn has_children(&self) -> bool {
        self.raw.children.is_some()
    }
    fn get_children_field(&self) -> TokenStream {
        if self.has_children() {
            let children_type_name = format_ident!("{}", self.children_struct_name());
            quote! {
                #[rkyv(omit_bounds)]
                #[tracked]
                pub children: Vec<#children_type_name<'db>>,
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
        let trait_impls = self.get_trait_implementations();

        quote! {
            #[salsa::tracked]
            // #[derive(Deserialize, Archive, Serialize, Drive)]
            // #serialize_bounds
            pub struct #name<'db> {
                #[drive(skip)]
                #[tracked]
                start_byte: usize,
                #[drive(skip)]
                #[tracked]
                end_byte: usize,
                #[drive(skip)]
                #[tracked]
                _kind: std::string::String,
                #[debug("[{},{}]", start_position.row, start_position.column)]
                #[drive(skip)]
                #[tracked]
                start_position: Point<'db>,
                #[debug("[{},{}]", end_position.row, end_position.column)]
                #[drive(skip)]
                #[tracked]
                end_position: Point<'db>,
                #[debug(ignore)]
                #[drive(skip)]
                #[tracked]
                buffer: Arc<Bytes>,
                #[debug(ignore)]
                #[drive(skip)]
                #[tracked]
                kind_id: u16,
                #[debug(ignore)]
                #[drive(skip)]
                #[tracked]
                is_error: bool,
                #[debug(ignore)]
                #[drive(skip)]
                #[tracked]
                named: bool,
                #[debug(ignore)]
                #[drive(skip)]
                #[tracked]
                id: usize,
                #children_field
                #(#struct_fields),*
            }
            #constructor
            #trait_impls
        }
    }
    fn get_children_constructor(&self) -> TokenStream {
        if self.has_children() {
            quote! {
                named_children_without_field_names(db,node, buffer)?
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
            impl<'db> FromNode<'db> for #name<'db> {
                fn from_node(db: &'db dyn salsa::Database, node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                    let start_position = Point::from(db, node.start_position());
                    let end_position = Point::from(db, node.end_position());
                    Ok(Self::new(db,
                        node.start_byte(),
                        node.end_byte(),
                        node.kind().to_string(),
                        start_position,
                        end_position,
                        buffer.clone(),
                        node.kind_id(),
                        node.is_error(),
                        node.is_named(),
                        node.id(),
                        #(#constructor_fields),*
                    ))
                }
            }
        }
    }
    fn get_children_impl(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let children_type_name = format_ident!("{}", self.children_struct_name());
        let children_field = self.get_children_field_impl();
        let children_by_field_name = self.get_children_by_field_name_impl();
        let children_by_field_id = self.get_children_by_field_id_impl();
        quote! {
            impl<'db> HasChildren<'db> for #name<'db> {
                type Child = #children_type_name;
                #children_field
                #children_by_field_name
                #children_by_field_id
            }
        }
    }
    pub fn get_trait_implementations(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let children_impl = self.get_children_impl();

        quote! {
            #[salsa::tracked]
            impl<'db> CSTNode<'db> for #name<'db> {
                #[salsa::tracked(return_ref)]
                fn kind(self, db: &'db dyn salsa::Database) -> std::string::String {
                    self._kind(db)
                }
                #[salsa::tracked]
                fn start_byte(self, db: &'db dyn salsa::Database) -> usize {
                    self.start_byte(db)
                }
                #[salsa::tracked]
                fn end_byte(self, db: &'db dyn salsa::Database) -> usize {
                    self.end_byte(db)
                }
                #[salsa::tracked]
                fn start_position(self, db: &'db dyn salsa::Database) -> Point<'db> {
                    self.start_position(db)
                }
                #[salsa::tracked]
                fn end_position(self, db: &'db dyn salsa::Database) -> Point<'db> {
                    self.end_position(db)
                }
                #[salsa::tracked(return_ref)]
                fn buffer(self, db: &'db dyn salsa::Database) -> Bytes {
                    (*self.buffer(db)).clone()
                }
                #[salsa::tracked]
                fn kind_id(self, db: &'db dyn salsa::Database) -> u16 {
                    self.kind_id(db)
                }
                #[salsa::tracked]
                fn is_error(self, db: &'db dyn salsa::Database) -> bool {
                    self.is_error(db)
                }
                #[salsa::tracked]
                fn is_named(self, db: &'db dyn salsa::Database) -> bool {
                    self.named(db)
                }
                #[salsa::tracked]
                fn id(self, db: &'db dyn salsa::Database) -> usize {
                    self.id(db)
                }
            }
            #children_impl
        }
    }
    fn get_children_field_impl(&self) -> TokenStream {
        let mut children_fields = Vec::new();
        let num_children = self.get_children_names().len();
        if num_children == 0 {
            return quote! {
                fn children(&self, db: &'db dyn salsa::Database) -> Vec<Self::Child> {
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
                self.children(db).iter().cloned().collect()
            }
        } else {
            quote! {
                vec![]
            }
        };
        quote! {
            fn children(&self, db: &'db dyn salsa::Database) -> Vec<Self::Child> {
                let mut children: Vec<_> = #children_init;
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
            fn children_by_field_name(&self, db: &'db dyn salsa::Database, field_name: &str) -> Vec<Self::Child> {
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
            fn children_by_field_id(&self, db: &'db dyn salsa::Database, field_id: u16) -> Vec<Self::Child> {
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
        let mut node = Node::new(&base_node, &language);

        let tokens = node.get_enum_tokens();
        insta::assert_debug_snapshot!(snapshot_tokens(&tokens));

        node.add_subenum("subenum".to_string());
        let tokens = node.get_enum_tokens();
        insta::assert_debug_snapshot!(snapshot_tokens(&tokens));
    }

    #[test]
    fn test_get_struct_tokens_simple() {
        let raw_node = create_test_node("test_node");
        let language = get_language_no_nodes();
        let node = Node::new(&raw_node, &language);
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens()));
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
        let node = Node::new(&raw_node, &language);
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens()));
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
        let node = Node::new(&raw_node, &language);
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens()));
    }

    #[test_log::test]
    fn test_get_struct_tokens_with_children() {
        let raw_node =
            create_test_node_with_children("test_node", vec!["child_type_a", "child_type_b"]);
        let language = get_language_no_nodes();
        let node = Node::new(&raw_node, &language);
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens()));
    }

    #[test]
    fn test_get_struct_tokens_with_single_child_type() {
        let raw_node = create_test_node_with_children("test_node", vec!["child_type"]);
        let language = get_language_no_nodes();
        let node = Node::new(&raw_node, &language);
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_struct_tokens()));
    }

    #[test]
    fn test_get_trait_implementations() {
        let raw_node = create_test_node("test_node");
        let language = get_language_no_nodes();
        let node = Node::new(&raw_node, &language);
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_trait_implementations()));
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
        let node = Node::new(&raw_node, &language);
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
        let node = Node::new(&raw_node, &language);
        insta::assert_debug_snapshot!(snapshot_tokens(&node.get_children_by_field_name_impl()));
    }
}
