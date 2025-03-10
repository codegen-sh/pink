use std::{hash::Hash, num::NonZeroU16, path::PathBuf, sync::Arc};

use convert_case::{Case, Casing};
use mockall::automock;
use proc_macro2::Span;
use tree_sitter::Parser;

use crate::{
    errors::ParseError,
    naming::normalize_type_name,
    parser::{Node, parse_node_types},
};
#[derive(Debug, Eq, PartialEq)]
pub struct Language {
    name: &'static str,
    pub struct_name: &'static str,
    pub node_types: &'static str,
    pub file_extensions: &'static [&'static str],
    tree_sitter_language: tree_sitter::Language,
    pub tag_query: &'static str,
    nodes: Vec<Arc<Node>>,
}
impl Hash for Language {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[automock]
impl Language {
    pub fn new(
        name: &'static str,
        struct_name: &'static str,
        node_types: &'static str,
        file_extensions: &'static [&'static str],
        tree_sitter_language: tree_sitter::Language,
        tag_query: &'static str,
    ) -> anyhow::Result<Self> {
        let nodes = parse_node_types(node_types)?
            .into_iter()
            .map(|node| Arc::new(node))
            .collect();
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
    pub fn nodes(&self) -> &Vec<Arc<Node>> {
        &self.nodes
    }
    pub fn root_node(&self) -> String {
        self.nodes()
            .iter()
            .find(|node| node.root)
            .unwrap_or_else(|| panic!("No root node found for language: {}", self.name))
            .type_name
            .to_case(Case::Pascal)
    }
    pub fn kind_id(&self, name: &str, named: bool) -> u16 {
        self.tree_sitter_language.id_for_node_kind(name, named)
    }
    pub fn kind_name(&self, id: u16) -> Option<&'static str> {
        self.tree_sitter_language.node_kind_for_id(id)
    }
    pub fn field_id(&self, name: &str) -> Option<NonZeroU16> {
        self.tree_sitter_language.field_id_for_name(name)
    }
    pub fn field_name(&self, id: u16) -> Option<&'static str> {
        self.tree_sitter_language.field_name_for_id(id)
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn struct_name(&self) -> &'static str {
        self.struct_name
    }
    pub fn file_struct_name(&self) -> syn::Ident {
        syn::Ident::new(&format!("{}File", self.struct_name()), Span::call_site())
    }
    pub fn package_name(&self) -> String {
        format!("codegen_sdk_{}", self.name())
    }
    pub fn node_for_struct_name(&self, struct_name: &str) -> Option<Arc<Node>> {
        self.nodes
            .iter()
            .find(|node| normalize_type_name(&node.type_name, node.named) == struct_name)
            .cloned()
    }
    pub fn should_parse(&self, file_path: &PathBuf) -> Result<bool, ParseError> {
        Ok(self.file_extensions.contains(
            &file_path
                .extension()
                .ok_or(ParseError::Miscelaneous)?
                .to_str()
                .ok_or(ParseError::Miscelaneous)?,
        ))
    }
}
#[cfg(feature = "go")]
pub mod go;
#[cfg(feature = "java")]
pub mod java;
#[cfg(feature = "typescript")]
pub mod javascript;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "typescript")]
pub mod jsx;
#[cfg(feature = "markdown")]
pub mod markdown;
#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "ruby")]
pub mod ruby;
#[cfg(feature = "rust")]
pub mod rust;
#[cfg(feature = "toml")]
pub mod toml;
#[cfg(feature = "ts_query")]
pub mod ts_query;
#[cfg(feature = "typescript")]
pub mod tsx;
#[cfg(feature = "typescript")]
pub mod typescript;
#[cfg(feature = "yaml")]
pub mod yaml;
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
        #[cfg(feature = "rust")]
        &rust::Rust,
        #[cfg(feature = "go")]
        &go::Go,
        #[cfg(feature = "ruby")]
        &ruby::Ruby,
        #[cfg(feature = "yaml")]
        &yaml::Yaml,
        #[cfg(feature = "toml")]
        &toml::TOML,
        #[cfg(feature = "markdown")]
        &markdown::Markdown,
        #[cfg(feature = "json")]
        &json::JSON,
        #[cfg(feature = "java")]
        &java::Java,
        #[cfg(feature = "ts_query")]
        &ts_query::Query,
    ];
}
