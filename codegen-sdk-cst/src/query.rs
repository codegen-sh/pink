use std::collections::HashMap;

use codegen_sdk_common::{CSTNode, Language, naming::normalize_type_name, HasChildren};
use derive_more::Debug;

use crate::{CSTLanguage, ts_query};
fn captures_for_field_definition(
    node: &ts_query::FieldDefinition,
) -> impl Iterator<Item = &ts_query::Capture> {
    let mut captures = Vec::new();
    for child in node.children() {
        match child {
            ts_query::FieldDefinitionChildren::NamedNode(named) => {
                captures.extend(captures_for_named_node(named));
            }
            ts_query::FieldDefinitionChildren::FieldDefinition(field) => {
                captures.extend(captures_for_field_definition(field));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
fn captures_for_named_node(node: &ts_query::NamedNode) -> impl Iterator<Item = &ts_query::Capture> {
    let mut captures = Vec::new();
    for child in node.children() {
        match child {
            ts_query::NamedNodeChildren::Capture(capture) => captures.push(capture),
            ts_query::NamedNodeChildren::NamedNode(named) => {
                captures.extend(captures_for_named_node(named));
            }
            ts_query::NamedNodeChildren::FieldDefinition(field) => {
                captures.extend(captures_for_field_definition(field));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
fn captures_for_node(node: &ts_query::Definition) -> impl Iterator<Item = &ts_query::Capture> {
    let mut captures = Vec::new();
    match node {
        ts_query::Definition::NamedNode(named) => captures.extend(captures_for_named_node(named)),
        ts_query::Definition::FieldDefinition(field) => {
            captures.extend(captures_for_field_definition(field))
        }
        _ => {}
    }
    captures.into_iter()
}
#[derive(Debug)]
#[debug("{}", self.source())]
pub struct Query {
    node: ts_query::NamedNode,
}
impl Query {
    pub fn from_queries(source: &str) -> HashMap<String, Self> {
        let parsed = ts_query::Query::parse(source).unwrap();
        let mut queries = HashMap::new();
        for node in parsed.children() {
            match node {
                ts_query::Definition::NamedNode(named) => {
                    let query = Self::from_named_node(named);
                    queries.insert(query.name(), query);
                }
                node => {
                    println!("Unhandled query: {:#?}", node);
                }
            }
        }

        // let root_node: Node<'a> = tree.root_node();
        // for child in root_node.children(&mut root_node.walk()) {
        //     if child.kind() == "grouping" {
        //         continue;
        //         // TODO: Handle grouping
        //     }
        //     let query = Self {
        //         query: child,
        //         source: source.to_string(),
        //     };
        //     queries.insert(query.name(), query);
        // }
        queries
    }
    fn from_named_node(named: &ts_query::NamedNode) -> Self {
        Query {
            node: named.clone(),
        }
    }
    /// Get the kind of the query (the node to be matched)
    pub fn kind(&self) -> String {
        if let ts_query::Types::Identifier(identifier) = &(*self.node.name) {
            return identifier.source();
        }
        panic!("No kind found for query. {:#?}", self.node);
    }
    pub fn struct_name(&self) -> String {
        normalize_type_name(&self.kind())
    }

    fn captures(&self) -> Vec<&ts_query::Capture> {
        captures_for_named_node(&self.node).collect()
    }
    /// Get the name of the query (IE @reference.class)
    pub fn name(&self) -> String {
        let mut result = self.captures().last().unwrap().source();
        result.replace_range(0..1, "");
        result
    }

    //     for node in self.query.named_children(&mut self.query.walk()) {
    //         for node in node.named_children(&mut self.query.walk()) {
    //             if node.kind() == "capture" {
    //                 return get_text_from_node(&node, &self.source);
    //             }
    //         }
    //     }

    //     panic!(
    //         "No name found for query. {:?}\n{}",
    //         self.query,
    //         self.source()
    //     );
    // }

    pub fn source(&self) -> String {
        self.node.source()
    }
}

pub trait HasQuery {
    fn queries(&self) -> HashMap<String, Query>;
    fn queries_with_prefix(&self, prefix: &str) -> HashMap<String, Query> {
        let mut queries = HashMap::new();
        for (name, query) in self.queries().into_iter() {
            if name.starts_with(prefix) {
                let new_name = name.split(".").last().unwrap();
                queries.insert(new_name.to_string(), query);
            }
        }
        queries
    }
    fn definitions(&self) -> HashMap<String, Query> {
        self.queries_with_prefix("definition")
    }
    fn references(&self) -> HashMap<String, Query> {
        self.queries_with_prefix("reference")
    }
}
impl HasQuery for Language {
    fn queries(&self) -> HashMap<String, Query> {
        Query::from_queries(&self.tag_query)
    }
}
