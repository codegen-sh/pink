use std::{collections::HashMap, fmt::Debug};

use tree_sitter::{Language, Node, TextProvider, Tree};

use crate::tree_sitter::get_text_from_node;
pub(crate) fn parse_query_tree(source: &str) -> anyhow::Result<tree_sitter::Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&Language::new(tree_sitter_query::LANGUAGE))?;
    Ok(parser.parse(source, None).unwrap())
}
pub struct Query<'a> {
    query: tree_sitter::Node<'a>,
    source: String,
}
impl<'a> Debug for Query<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Query: {}\n{}", self.name(), self.source())
    }
}
impl<'a> Query<'a> {
    pub fn from_queries(tree: &'a Tree, source: &str) -> HashMap<String, Self> {
        let root_node: Node<'a> = tree.root_node();
        let mut queries = HashMap::new();
        for child in root_node.children(&mut root_node.walk()) {
            if child.kind() == "grouping" {
                continue;
                // TODO: Handle grouping
            }
            let query = Self {
                query: child,
                source: source.to_string(),
            };
            queries.insert(query.name(), query);
        }
        queries
    }
    /// Get the kind of the query (the node to be matched)
    pub fn kind(&self) -> String {
        let name_node = self.query.child_by_field_name("name").unwrap();
        get_text_from_node(&name_node, &self.source)
    }
    /// Get the name of the query (IE @reference.class)
    pub fn name(&self) -> String {
        for node in self.query.named_children(&mut self.query.walk()) {
            if node.kind() == "capture" {
                return get_text_from_node(&node, &self.source);
            }
        }

        for node in self.query.named_children(&mut self.query.walk()) {
            for node in node.named_children(&mut self.query.walk()) {
                if node.kind() == "capture" {
                    return get_text_from_node(&node, &self.source);
                }
            }
        }

        panic!(
            "No name found for query. {:?}\n{}",
            self.query,
            self.source()
        );
    }

    fn source(&self) -> String {
        get_text_from_node(&self.query, &self.source)
    }
}
