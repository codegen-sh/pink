use std::collections::BTreeMap;

use codegen_sdk_common::{naming::normalize_type_name, parser::TypeDefinition};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::constants::TYPE_NAME;
pub fn get_serialize_bounds() -> TokenStream {
    quote! {
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
    }
}
pub fn get_from_type(struct_name: &str) -> TokenStream {
    let name = format_ident!("{}", struct_name);
    let target = format_ident!("{}", TYPE_NAME);
    quote! {
        impl<'db> From<#name<'db>> for #target<'db> {
            fn from(node: #name<'db>) -> Self {
                Self::#name(node)
            }
        }
    }
}
pub fn get_from_node(
    node: &str,
    named: bool,
    variant_map: &BTreeMap<u16, TokenStream>,
) -> TokenStream {
    let node = format_ident!("{}", normalize_type_name(node, named));
    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in variant_map.iter() {
        keys.push(key);
        values.push(value);
    }
    quote! {
        impl<'db> FromNode<'db> for #node<'db> {
            fn from_node(db: &'db dyn salsa::Database, node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                match node.kind_id() {
                    #(#keys => #values,)*
                    _ => Err(ParseError::UnexpectedNode {
                        node_type: node.kind().to_string(),
                        backtrace: Backtrace::capture(),
                    }),                }
            }
        }
    }
}
pub fn get_comment_type() -> TypeDefinition {
    TypeDefinition {
        type_name: "comment".to_string(),
        named: true,
    }
}
