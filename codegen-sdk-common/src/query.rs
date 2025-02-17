use std::collections::HashMap;

use tree_sitter::{Language, Node, TextProvider, Tree};
pub(crate) fn parse_query_tree(source: &str) -> anyhow::Result<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&Language::new(tree_sitter_query::LANGUAGE))?;
    Ok(parser.parse(source, None).unwrap())
}
#[derive(Debug)]
pub struct Query<'a> {
    query: tree_sitter::Node<'a>,
    source: String,
}
impl<'a> Query<'a> {
    pub fn from_queries(tree: &'a Tree, source: &str) -> HashMap<String, Self> {
        let root_node: Node<'a> = tree.root_node();
        let mut queries = HashMap::new();
        for child in root_node.children(&mut root_node.walk()) {
            let query = Self {
                query: child,
                source: source.to_string(),
            };
            queries.insert(query.name(), query);
        }
        queries
    }
    pub fn name(&self) -> String {
        let name_node = self.query.child_by_field_name("name").unwrap();
        String::from_utf8(
            self.source
                .as_bytes()
                .text(name_node)
                .next()
                .unwrap()
                .to_vec(),
        )
        .unwrap()
    }
}
