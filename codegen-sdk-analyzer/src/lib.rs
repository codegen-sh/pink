#![recursion_limit = "512"]
mod database;
mod parser;
mod progress;
pub use parser::{Parsed, ParsedFile};
mod codebase;
pub use codebase::Codebase;
