#![recursion_limit = "512"]
mod database;
mod parser;
mod progress;
pub use database::{CodegenDatabase, Db};
pub use parser::{Parsed, ParsedFile, parse_file};
