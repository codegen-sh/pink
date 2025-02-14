use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("TreeSitter error: {0}")]
    TreeSitter(#[from] tree_sitter::LanguageError),
    #[error("Unknown Language")]
    UnknownLanguage,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Miscelaneous error")]
    Miscelaneous,
}
