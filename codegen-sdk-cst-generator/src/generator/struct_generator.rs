use codegen_sdk_common::{
    naming::{normalize_field_name, normalize_type_name},
    parser::{FieldDefinition, Fields, Node, TypeDefinition},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use super::enum_generator::generate_enum;
use crate::generator::state::State;
struct FieldOutput {
    struct_field: TokenStream,
    constructor_field: TokenStream,
    children_field: TokenStream,
    children_by_field_name_field: TokenStream,
    field_name: String,
    field_name_identifier: Ident,
}
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
) -> FieldOutput {
    let field_name_ident = format_ident!("{}", field_name);
    let converted_type_name = format_ident!("{}", converted_type_name);
    let struct_field = quote! {
           pub #field_name_ident: Vec<#converted_type_name>
    };
    let constructor_field = quote! {
        #field_name_ident: get_multiple_children_by_field_name(&node, #original_name, buffer)?
    };
    let convert_child = quote! {
        &((*child).clone().into())
    };
    FieldOutput {
        struct_field,
        constructor_field,
        children_field: quote! {
            children.extend(self.#field_name_ident.iter().map(#convert_child));
        },
        children_by_field_name_field: quote! {
            #field_name => self.#field_name_ident.iter().map(#convert_child).collect()
        },
        field_name: field_name.to_string(),
        field_name_identifier: field_name_ident,
    }
}
fn generate_required_field(
    field_name: &str,
    converted_type_name: &str,
    original_name: &str,
) -> FieldOutput {
    let field_name_ident = format_ident!("{}", field_name);
    let converted_type_name = format_ident!("{}", converted_type_name);
    let struct_field = quote! {
        #[rkyv(omit_bounds)]
        pub #field_name_ident: Box<#converted_type_name>
    };
    let constructor_field = quote! {
        #field_name_ident: Box::new(get_child_by_field_name(&node, #original_name, buffer)?)
    };
    let convert_child = quote! {
        &((*self.#field_name_ident).clone().into())
    };
    FieldOutput {
        struct_field,
        constructor_field,
        children_field: quote! {
            children.push(#convert_child);
        },
        children_by_field_name_field: quote! {
            #field_name => vec![#convert_child]
        },
        field_name: field_name.to_string(),
        field_name_identifier: field_name_ident,
    }
}
fn generate_optional_field(
    field_name: &str,
    converted_type_name: &str,
    original_name: &str,
) -> FieldOutput {
    let field_name_ident = format_ident!("{}", field_name);
    let converted_type_name = format_ident!("{}", converted_type_name);
    let struct_field = quote! {
        #[rkyv(omit_bounds)]
        pub #field_name_ident: Box<Option<#converted_type_name>>
    };
    let constructor_field = quote! {
        #field_name_ident: Box::new(get_optional_child_by_field_name(&node, #original_name, buffer)?)
    };
    let convert_child = quote! {
        &((*child).clone().into())
    };
    FieldOutput {
        struct_field,
        constructor_field,
        children_field: quote! {
            if let Some(child) = self.#field_name_ident {
                children.push(#convert_child);
            }
        },
        children_by_field_name_field: quote! {
            #field_name => self.#field_name_ident.map_or_else(|| vec![], |child| vec![#convert_child])
        },
        field_name: field_name.to_string(),
        field_name_identifier: field_name_ident,
    }
}
fn generate_field(
    field: &FieldDefinition,
    state: &mut State,
    node: &Node,
    original_name: &str,
    field_name: &str,
) -> FieldOutput {
    let converted_type_name =
        convert_type_definition(&field.types, state, &node.type_name, original_name);
    if field.multiple {
        return generate_multiple_field(
            &field_name,
            &converted_type_name,
            original_name,
        );
    } else if field.required {
        return generate_required_field(
            &field_name,
            &converted_type_name,
            original_name,
        );
    } else {
        return generate_optional_field(
            &field_name,
            &converted_type_name,
            original_name,
        );
    }
}
fn generate_fields(fields: &Fields, state: &mut State, node: &Node) -> Vec<FieldOutput> {
    let mut field_outputs = Vec::new();
    for (name, field) in &fields.fields {
        let field_name = normalize_field_name(name);
        let field_output = generate_field(field, state, node, name, &field_name);
        field_outputs.push(field_output);
    }
    field_outputs
}
fn generate_children(
    children: &Vec<TypeDefinition>,
    state: &mut State,
    node_name: &str,
) -> (String, TokenStream) {
    let converted_type_name = convert_type_definition(children, state, node_name, "children");
    let constructor_field = quote! {
        children: named_children_without_field_names(node, buffer)?
    };
    (converted_type_name, constructor_field)
}
fn generate_children_field(children_fields: &Vec<TokenStream>) -> TokenStream {
    let m = if children_fields.is_empty() {
        quote! {}
    } else {
        quote! {mut}
    };
    quote! {
        fn children(&self) -> Vec<&Self::Child> {
            let #m children: Vec<_> = self.children.iter().collect();
            #(#children_fields;)*
            children
         }
    }
}
fn collect_children(node: &Node) -> Vec<TypeDefinition> {
    let mut children = if let Some(children) = &node.children {
        children.types.clone()
    } else {
        vec![]
    };
    if let Some(fields) = &node.fields {
        for field in fields.fields.values() {
            children.extend(field.types.clone());
        }
    }
    children.sort_by_key(|t| t.type_name.clone());
    children.dedup();
    children
}

pub fn generate_struct(node: &Node, state: &mut State, name: &str) {
    let mut constructor_fields = Vec::new();
    let mut struct_fields = Vec::new();
    let mut field_names = Vec::new();
    let mut children_fields = Vec::new();
    let mut children_by_field_name_fields = Vec::new();
    let mut children_type_name = "Self".to_string();
    let children = collect_children(node);
    if !children.is_empty() {
        let constructor_field;
        (children_type_name, constructor_field) =
            generate_children(&children, state, &node.type_name);
        constructor_fields.push(constructor_field);
    } else {
        constructor_fields.push(quote! {
            children: vec![]
        });
    }
    let children_type_name = format_ident!("{}", children_type_name);
    if let Some(fields) = &node.fields {
        for field_output in generate_fields(fields, state, node) {
            struct_fields.push(field_output.struct_field);
            constructor_fields.push(field_output.constructor_field);
            field_names.push(field_output.field_name);
            children_fields.push(field_output.children_field);
            children_by_field_name_fields.push(field_output.children_by_field_name_field);
        }
    }
    let children_field = generate_children_field(&children_fields);
    let name = format_ident!("{}", name);
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
        pub children: Vec<#children_type_name>,
        #(#struct_fields),*
    }
    };
    state.structs.extend_one(definition);
    let implementation = quote! {
        impl CSTNode for #name {
            type Child = #children_type_name;
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
            #children_field
            fn children_by_field_name(&self, field_name: &str) -> Vec<&Self::Child> {
                match field_name {
                    #(#children_by_field_name_fields,)*
                    _ => vec![],
                }
            }
        }
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
    };
    state.structs.extend_one(implementation);
}
