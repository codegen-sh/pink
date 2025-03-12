#![recursion_limit = "512"]
mod database;
mod parser;
mod progress;
use codegen_sdk_macros::re_export_languages;
pub use parser::{Parsed, ParsedFile, parse_file};
mod codebase;
pub use codebase::Codebase;
re_export_languages!();
