use std::backtrace::Backtrace;

use salsa::Accumulator;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("TreeSitter error: {0}")]
    TreeSitter(#[from] tree_sitter::LanguageError),
    #[error("Syntax error")]
    SyntaxError,
    #[error("Unknown Language")]
    UnknownLanguage,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(
        "Missing Required Field '{field_name}' in node of type '{parent_node}' with backtrace:\n {backtrace}"
    )]
    MissingNode {
        field_name: String,
        parent_node: String,
        backtrace: Backtrace,
    },
    #[error("Miscelaneous error")]
    Miscelaneous,
    #[error("Unexpected Node Type {node_type} with backtrace:\n {backtrace}")]
    UnexpectedNode {
        node_type: String,
        backtrace: Backtrace,
    },
    #[error("Failed to serialize: {0}")]
    Serialize(#[from] rkyv::rancor::Error),
}
#[salsa::accumulator]
#[allow(dead_code)] // Debug impl uses them
struct AccumulatedParseError {
    message: String,
}
impl ParseError {
    pub fn report(self, db: &dyn salsa::Database) {
        AccumulatedParseError {
            message: self.to_string(),
        }
        .accumulate(db);
    }
}
