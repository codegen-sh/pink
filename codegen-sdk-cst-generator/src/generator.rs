use std::collections::HashSet;

use codegen_sdk_common::{naming::normalize_type_name, parser::Node};
use enum_generator::generate_enum;
use state::State;
use struct_generator::generate_struct;
mod constants;
mod enum_generator;
mod field;
mod format;
mod node;
mod state;
mod struct_generator;
mod utils;
use std::io::Write;

use proc_macro2::TokenStream;
use quote::quote;
fn get_imports() -> TokenStream {
    quote! {

    use std::sync::Arc;
    use tree_sitter;
    use derive_more::Debug;
    use codegen_sdk_common::*;
    use subenum::subenum;
    use std::backtrace::Backtrace;
    use bytes::Bytes;
    use rkyv::{Archive, Deserialize, Serialize};

        }
}
pub fn generate_cst(node_types: &Vec<Node>) -> anyhow::Result<String> {
    let mut state = State::from(node_types);
    let mut nodes = HashSet::new();
    let mut enums = Vec::new();

    for node in node_types {
        if !node.subtypes.is_empty() {
            let name = normalize_type_name(&node.type_name, node.named);
            enums.push(quote! {
                #name(#name)
            });
            state.variants.insert(name, node.subtypes.clone());
        } else if node.children.is_none() && node.fields.is_none() {
            state.anonymous_nodes.insert(
                node.type_name.clone(),
                normalize_type_name(&node.type_name, node.named),
            );
        }
    }
    for node in node_types {
        let name = normalize_type_name(&node.type_name, node.named);
        if nodes.contains(&name) {
            continue;
        }
        nodes.insert(name.clone());
        if name.is_empty() {
            continue;
        }
        if !node.subtypes.is_empty() {
            generate_enum(&node.subtypes, &mut state, &name, false);
        } else {
            generate_struct(node, &mut state, &name);
        }
    }
    let mut result = get_imports();
    let enums = state.get_enum();
    let structs = state.get_structs();
    // result.extend_one(state.enums);
    // result.extend_one(state.structs);
    result.extend_one(enums);
    result.extend_one(structs);
    let formatted = format::format_cst(&result.to_string());
    match formatted {
        Ok(formatted) => return Ok(formatted),
        Err(e) => {
            let mut out_file = tempfile::NamedTempFile::with_suffix(".rs")?;
            log::error!(
                "Failed to format CST, writing to temp file at {}",
                out_file.path().display()
            );
            out_file.write_all(result.to_string().as_bytes())?;
            out_file.keep()?;
            return Err(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use codegen_sdk_common::{language::python::Python, parser::parse_node_types};

    use super::*;
    #[test_log::test]
    fn test_generate_cst() {
        let node_types = parse_node_types(&Python.node_types).unwrap();
        let cst = generate_cst(&node_types).unwrap();
        log::info!("{}", cst);
    }
}
