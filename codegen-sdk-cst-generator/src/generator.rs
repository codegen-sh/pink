use codegen_sdk_common::parser::Node;
use state::State;
mod constants;
mod field;
mod format;
mod node;
mod state;
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
    let state = State::from(node_types);
    let mut result = get_imports();
    let enums = state.get_enum();
    let structs = state.get_structs();
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
