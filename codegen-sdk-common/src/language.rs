use crate::errors::ParseError;
use tree_sitter::Parser;
pub struct Language {
    pub name: &'static str,
    pub struct_name: &'static str,
    pub node_types: &'static str,
    pub file_extensions: &'static [&'static str],
    pub tree_sitter_language: tree_sitter::Language,
}
impl Language {
    pub fn parse_tree_sitter(&self, content: &str) -> Result<tree_sitter::Tree, ParseError> {
        let mut parser = Parser::new();
        parser.set_language(&self.tree_sitter_language)?;
        parser.parse(content, None).ok_or(ParseError::Miscelaneous)
    }
}
#[cfg(feature = "java")]
pub mod java;
#[cfg(feature = "typescript")]
pub mod javascript;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "typescript")]
pub mod jsx;
#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "typescript")]
pub mod tsx;
#[cfg(feature = "typescript")]
pub mod typescript;
lazy_static! {
    pub static ref LANGUAGES: Vec<&'static Language> = vec![
        #[cfg(feature = "python")]
        &python::Python,
        #[cfg(feature = "typescript")]
        &typescript::Typescript,
        #[cfg(feature = "typescript")]
        &tsx::TSX,
        #[cfg(feature = "typescript")]
        &jsx::JSX,
        #[cfg(feature = "typescript")]
        &javascript::Javascript,
        #[cfg(feature = "json")]
        &json::JSON,
        #[cfg(feature = "java")]
        &java::Java,
    ];
}
