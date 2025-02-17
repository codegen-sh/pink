use std::collections::HashMap;

use codegen_sdk_common::Language;

use crate::CSTLanguage;
#[derive(Debug)]
pub struct Query {}
impl Query {
    pub fn from_queries(source: &str) -> HashMap<String, Self> {
        let parsed = crate::ts_query::Query::parse(source).unwrap();
        println!("{:#?}", parsed);

        // let root_node: Node<'a> = tree.root_node();
        let mut queries = HashMap::new();
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
    // /// Get the kind of the query (the node to be matched)
    // pub fn kind(&self) -> String {
    //     let name_node = self.query.child_by_field_name("name").unwrap();
    //     get_text_from_node(&name_node, &self.source)
    // }
    // /// Get the name of the query (IE @reference.class)
    // pub fn name(&self) -> String {
    //     for node in self.query.named_children(&mut self.query.walk()) {
    //         if node.kind() == "capture" {
    //             return get_text_from_node(&node, &self.source);
    //         }
    //     }

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

    // fn source(&self) -> String {
    //     get_text_from_node(&self.query, &self.source)
    // }
}

pub trait HasQuery {
    fn queries(&self) -> HashMap<String, Query>;
}
impl HasQuery for Language {
    fn queries(&self) -> HashMap<String, Query> {
        Query::from_queries(&self.tag_query)
    }
}
