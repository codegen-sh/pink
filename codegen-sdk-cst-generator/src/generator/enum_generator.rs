// TODO: migrate to use struct based generation
use codegen_sdk_common::{
    naming::{normalize_string, normalize_type_name},
    parser::TypeDefinition,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::generator::state::State;
fn get_cases(
    variants: &Vec<TypeDefinition>,
    state: &State,
    override_variant_name: Option<&str>,
    existing_cases: &mut Vec<String>,
) -> Vec<(String, TokenStream)> {
    let mut cases = Vec::new();
    for t in variants {
        let normalized_variant_name = normalize_type_name(&t.type_name);
        if normalized_variant_name.is_empty() {
            continue;
        }
        let variant_name = override_variant_name.unwrap_or_else(|| &normalized_variant_name);
        if let Some(variants) = state.variants.get(&normalized_variant_name) {
            cases.extend(get_cases(
                variants,
                state,
                Some(variant_name),
                existing_cases,
            ));
        } else if !existing_cases.contains(&t.type_name) {
            existing_cases.push(t.type_name.clone());
            let variant_name = format_ident!("{}", normalized_variant_name);
            cases.push((
                t.type_name.clone(),
                quote! { Self::#variant_name (#variant_name::from_node(node, buffer)?)},
            ));
            // cases.insert(t.type_name.clone(), quote!{
            //     #t.type_name => Ok(#(#prefix)::from_node(node, buffer)?),
            // }.to_string());
        }
    }
    return cases;
}
pub fn generate_enum(
    variants: &Vec<TypeDefinition>,
    state: &mut State,
    enum_name: &str,
    anonymous_nodes: bool,
) {
    let mut variant_tokens = Vec::new();
    let enum_name = format_ident!("{}", enum_name);
    for t in variants {
        let variant_name = normalize_type_name(&t.type_name);
        if variant_name.is_empty() {
            continue;
        }
        let variant_name = format_ident!("{}", variant_name);
        variant_tokens.push(quote! {
            #variant_name(#variant_name)
        });
        if state.subenums.contains(&t.type_name) {
            continue;
        }
        state.enums.extend_one(quote! {
            impl std::convert::From<#variant_name> for #enum_name {
                fn from(variant: #variant_name) -> Self {
                    Self::#variant_name(variant)
                }
            }
            // impl <T: std::convert::Into<#variant_name>> std::convert::From<T> for #enum_name {
            //     fn from(variant: T) -> Self {
            //         Self::#variant_name(variant.into())
            //     }
            // }
        });
    }
    if anonymous_nodes {
        variant_tokens.push(quote! {
            Anonymous,
        });
    }
    // state.enums.extend_one(quote! {
    //     #[derive(Debug, Clone, Archive, Portable, Deserialize, Serialize)]
    //     #[repr(C, u8)]
    //     pub enum #enum_name {
    //         #(#variant_tokens),*
    //     }
    // });
    let mut existing_cases = Vec::new();
    let mut cases = get_cases(variants, state, None, &mut existing_cases);
    if anonymous_nodes {
        for (name, _variant_name) in state.anonymous_nodes.iter() {
            if name.is_empty() {
                continue;
            }
            if existing_cases.contains(name) {
                continue;
            }
            let normalized_name = normalize_string(name);
            cases.push((normalized_name, quote! {Self::Anonymous}));
        }
    }
    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (key, value) in cases {
        keys.push(key);
        values.push(value);
    }
    state.enums.extend_one(quote! {
    impl FromNode for #enum_name {
        fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError> {
            match node.kind() {
                #(#keys => Ok(#values)),*,
                _ => Err(ParseError::UnexpectedNode {
                    node_type: node.kind().to_string(),
                    backtrace: Backtrace::capture(),
                }),
            }
        }
        }
    });
}
