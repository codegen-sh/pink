use codegen_sdk_common::{
    naming::{normalize_field_name, normalize_type_name},
    parser::{Children, FieldDefinition, Fields, Node, TypeDefinition},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::enum_generator::generate_enum;
use crate::generator::state::State;
fn convert_type_definition(
    type_name: &Vec<TypeDefinition>,
    state: &mut State,
    field_name: &str,
    node_name: &str,
) -> String {
    let include_anonymous_nodes = true;
    if type_name.len() == 1 && !include_anonymous_nodes {
        normalize_type_name(&type_name[0].type_name)
    } else {
        let enum_name = normalize_type_name(
            format!(
                "{}{}",
                normalize_type_name(node_name),
                normalize_type_name(field_name)
            )
            .as_str(),
        );
        generate_enum(type_name, state, &enum_name, include_anonymous_nodes);
        enum_name
    }
}

fn generate_multiple_field(
    field_name: &str,
    converted_type_name: &str,
    original_name: &str,
) -> (TokenStream, TokenStream) {
    let field_name = format_ident!("{}", field_name);
    let converted_type_name = format_ident!("{}", converted_type_name);
    let struct_field = quote! {
           pub #field_name: Vec<#converted_type_name>
    };
    let constructor_field = quote! {
        #field_name: get_multiple_children_by_field_name(&node, #original_name, buffer)?
    };
    (struct_field, constructor_field)
}
fn generate_required_field(
    field_name: &str,
    converted_type_name: &str,
    original_name: &str,
) -> (TokenStream, TokenStream) {
    let field_name = format_ident!("{}", field_name);
    let converted_type_name = format_ident!("{}", converted_type_name);
    let struct_field = quote! {
        #[rkyv(omit_bounds)]
        pub #field_name: Box<#converted_type_name>
    };
    let constructor_field = quote! {
        #field_name: Box::new(get_child_by_field_name(&node, #original_name, buffer)?)
    };
    (struct_field, constructor_field)
}
fn generate_optional_field(
    field_name: &str,
    converted_type_name: &str,
    original_name: &str,
) -> (TokenStream, TokenStream) {
    let field_name = format_ident!("{}", field_name);
    let converted_type_name = format_ident!("{}", converted_type_name);
    let struct_field = quote! {
        #[rkyv(omit_bounds)]
        pub #field_name: Box<Option<#converted_type_name>>
    };
    let constructor_field = quote! {
        #field_name: Box::new(get_optional_child_by_field_name(&node, #original_name, buffer)?)
    };
    (struct_field, constructor_field)
}
fn generate_field(
    field: &FieldDefinition,
    state: &mut State,
    node: &Node,
    name: &str,
) -> (TokenStream, TokenStream) {
    let field_name = normalize_field_name(name);
    let converted_type_name = convert_type_definition(&field.types, state, &node.type_name, name);
    if field.multiple {
        return generate_multiple_field(&field_name, &converted_type_name, name);
    } else if field.required {
        return generate_required_field(&field_name, &converted_type_name, name);
    } else {
        return generate_optional_field(&field_name, &converted_type_name, name);
    }
}
fn generate_fields(
    fields: &Fields,
    state: &mut State,
    node: &Node,
) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let mut struct_fields = Vec::new();
    let mut constructor_fields = Vec::new();
    for (name, field) in &fields.fields {
        let (struct_field, constructor_field) = generate_field(field, state, node, name);
        struct_fields.push(struct_field);
        constructor_fields.push(constructor_field);
    }
    (struct_fields, constructor_fields)
}
fn generate_children(
    children: &Children,
    state: &mut State,
    node_name: &str,
) -> (String, TokenStream) {
    let converted_type_name =
        convert_type_definition(&children.types, state, node_name, "children");
    let constructor_field = quote! {
        children: named_children_without_field_names(node, buffer)?
    };
    (converted_type_name, constructor_field)
}
pub fn generate_struct(node: &Node, state: &mut State, name: &str) {
    let mut constructor_fields = Vec::new();
    let mut struct_fields = Vec::new();
    if let Some(fields) = &node.fields {
        (struct_fields, constructor_fields) = generate_fields(fields, state, node);
    }
    let mut children_type_name = "Self".to_string();
    if let Some(children) = &node.children {
        let constructor_field;
        (children_type_name, constructor_field) =
            generate_children(children, state, &node.type_name);
        constructor_fields.push(constructor_field);
    } else {
        constructor_fields.push(quote! {
            children: vec![]
        });
    }
    let name = format_ident!("{}", name);
    let children_type_name = format_ident!("{}", children_type_name);
    let definition = quote! {
            #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
    #[rkyv(serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator,
        __S::Error: rkyv::rancor::Source,
    ))]
    #[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]
    #[rkyv(bytecheck(
        bounds(
            __C: rkyv::validation::ArchiveContext,
            __C::Error: rkyv::rancor::Source,
        )
    ))]
    pub struct #name {
        start_byte: usize,
        end_byte: usize,
        #[debug("[{},{}]", start_position.row, start_position.column)]
        start_position: Point,
        #[debug("[{},{}]", end_position.row, end_position.column)]
        end_position: Point,
        #[debug(ignore)]
        buffer: Arc<Bytes>,
        #[debug(ignore)]
        kind_id: u16,
        #[rkyv(omit_bounds)]
        pub children: Vec<#children_type_name>,
        #(#struct_fields),*
    }
    };
    state.structs.extend_one(definition);
    let implementation = quote! {
        impl CSTNode for #name {
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
        impl HasChildren for #name {
            type Child = #children_type_name;
            fn children(&self) -> &Vec<Self::Child> {
               self.children.as_ref()
            }
        }
        impl FromNode for #name {
            fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                Ok(Self {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_position: node.start_position().into(),
                    end_position: node.end_position().into(),
                    buffer: buffer.clone(),
                    kind_id: node.kind_id(),
                    #(#constructor_fields),*
                })
            }
        }
    };
    state.structs.extend_one(implementation);
}
