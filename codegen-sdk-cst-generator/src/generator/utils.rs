use std::collections::BTreeMap;

use codegen_sdk_common::naming::normalize_type_name;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
pub fn get_from_for_enum(variant: &str, enum_name: &str) -> TokenStream {
    let enum_name = format_ident!("{}", enum_name);
    let variant = format_ident!("{}", variant);
    quote! {
        impl std::convert::From<#variant> for #enum_name {
            fn from(variant: #variant) -> Self {
                Self::#variant(variant)
            }
        }
    }
}
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
pub fn get_from_node(node: &str, variant_map: &BTreeMap<String, TokenStream>) -> TokenStream {
    let node = format_ident!("{}", normalize_type_name(node));
    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in variant_map.iter() {
        keys.push(key);
        values.push(value);
    }
    quote! {
        impl FromNode for #node {
            fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
                match node.kind() {
                    #(#keys => #values,)*
                    _ => Err(ParseError::UnexpectedNode {
                        node_type: node.kind().to_string(),
                        backtrace: Backtrace::capture(),
                    }),                }
            }
        }
    }
}
