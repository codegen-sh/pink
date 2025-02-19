use std::collections::HashSet;

use codegen_sdk_common::{naming::normalize_type_name, parser::Node};
use enum_generator::generate_enum;
use state::State;
use struct_generator::generate_struct;
mod enum_generator;
mod format;
mod state;
mod struct_generator;
const IMPORTS: &str = "
use std::sync::Arc;
use tree_sitter;
use derive_more::Debug;
use codegen_sdk_common::*;
use std::backtrace::Backtrace;
use bytes::Bytes;
use rkyv::{Archive, Deserialize, Serialize, Portable};
";

pub(crate) fn generate_cst(node_types: &Vec<Node>) -> anyhow::Result<String> {
    let mut state = State::default();
    let mut nodes = HashSet::new();
    for node in node_types {
        if !node.subtypes.is_empty() {
            state
                .variants
                .insert(normalize_type_name(&node.type_name), node.subtypes.clone());
        } else if node.children.is_none() && node.fields.is_none() {
            state
                .anonymous_nodes
                .insert(node.type_name.clone(), normalize_type_name(&node.type_name));
        }
    }
    for node in node_types {
        let name = normalize_type_name(&node.type_name);
        if nodes.contains(&name) {
            continue;
        }
        nodes.insert(name.clone());
        if name.is_empty() {
            continue;
        }
        if !node.subtypes.is_empty() {
            generate_enum(&node.subtypes, &mut state, &name, true);
        } else {
            generate_struct(node, &mut state, &name);
        }
    }
    let mut result = IMPORTS.to_string();
    result.push_str(&state.enums.to_string());
    result.push_str(&state.structs);
    let formatted = format::format_cst(&result);
    match formatted {
        Ok(formatted) => return Ok(formatted),
        Err(e) => {
            log::error!("Failed to format CST: {}", e);
            return Ok(result.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use codegen_sdk_common::{language::python::Python, parser::parse_node_types};

    use super::*;
    #[test]
    fn test_generate_cst() {
        let node_types = parse_node_types(&Python.node_types).unwrap();
        let cst = generate_cst(&node_types).unwrap();
        log::info!("{}", cst);
    }
}
