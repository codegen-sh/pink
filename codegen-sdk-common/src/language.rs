use convert_case::{Case, Casing};
use tree_sitter::Parser;

use crate::{
    errors::ParseError,
    parser::{parse_node_types, Node},
};
pub struct Language {
    pub name: &'static str,
    pub struct_name: &'static str,
    pub node_types: &'static str,
    pub file_extensions: &'static [&'static str],
    pub tree_sitter_language: tree_sitter::Language,
    pub tag_query: &'static str,
    nodes: Vec<Node>,
}
impl Language {
    pub(crate) fn new(
        name: &'static str,
        struct_name: &'static str,
        node_types: &'static str,
        file_extensions: &'static [&'static str],
        tree_sitter_language: tree_sitter::Language,
        tag_query: &'static str,
    ) -> anyhow::Result<Self> {
        let nodes = parse_node_types(node_types)?;
        Ok(Self {
            name,
            struct_name,
            node_types,
            file_extensions,
            tree_sitter_language,
            tag_query,
            nodes,
        })
    }
    pub fn parse_tree_sitter(&self, content: &str) -> Result<tree_sitter::Tree, ParseError> {
        let mut parser = Parser::new();
        parser.set_language(&self.tree_sitter_language)?;
        parser.parse(content, None).ok_or(ParseError::Miscelaneous)
    }
    pub fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }
    pub fn root_node(&self) -> String {
        self.nodes()
            .iter()
            .find(|node| node.root)
            .unwrap()
            .type_name
            .to_case(Case::Pascal)
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
#[cfg(feature = "ts_query")]
pub mod ts_query;
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
        #[cfg(feature = "ts_query")]
        &ts_query::Query,
    ];
}
