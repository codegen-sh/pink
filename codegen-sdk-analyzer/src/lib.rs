#![recursion_limit = "512"]
mod database;
mod parser;
mod progress;
pub use parser::{Parsed, ParsedFile, parse_file};
mod codebase;
pub use codebase::Codebase;
