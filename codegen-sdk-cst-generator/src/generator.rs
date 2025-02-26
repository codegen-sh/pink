#[double]
use codegen_sdk_common::language::Language;
pub use field::Field;
use mockall_double::double;
pub use node::Node;
pub use state::State;
mod constants;
mod field;
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
    use derive_visitor::Drive;
    use delegation::delegate;

        }
}
pub fn generate_cst(language: &Language) -> anyhow::Result<String> {
    let state = State::new(language);
    let mut result = get_imports();
    let enums = state.get_enum();
    let structs = state.get_structs();
    result.extend_one(enums);
    result.extend_one(structs);
    let formatted = codegen_sdk_common::generator::format_code(&result.to_string());
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
