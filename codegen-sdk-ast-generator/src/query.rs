use std::{collections::BTreeMap, sync::Arc};

use codegen_sdk_common::{CSTNode, HasChildren, Language, naming::normalize_type_name};
use codegen_sdk_cst::{CSTLanguage, ts_query};
use codegen_sdk_cst_generator::{Field, State};
use derive_more::Debug;
use log::info;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
fn captures_for_field_definition(
    node: &ts_query::FieldDefinition,
) -> impl Iterator<Item = ts_query::Capture> {
    let mut captures = Vec::new();
    for child in node.children() {
        match child {
            ts_query::FieldDefinitionChildren::NamedNode(named) => {
                captures.extend(captures_for_named_node(&named));
            }
            ts_query::FieldDefinitionChildren::FieldDefinition(field) => {
                captures.extend(captures_for_field_definition(&field));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
fn captures_for_named_node(node: &ts_query::NamedNode) -> impl Iterator<Item = ts_query::Capture> {
    let mut captures = Vec::new();
    for child in node.children() {
        match child {
            ts_query::NamedNodeChildren::Capture(capture) => captures.push(capture),
            ts_query::NamedNodeChildren::NamedNode(named) => {
                captures.extend(captures_for_named_node(&named));
            }
            ts_query::NamedNodeChildren::FieldDefinition(field) => {
                captures.extend(captures_for_field_definition(&field));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
#[derive(Debug)]
#[debug("{}", self.source())]
pub struct Query<'a> {
    node: ts_query::NamedNode,
    language: &'a Language,
    state: Arc<State<'a>>,
}
impl<'a> Query<'a> {
    pub fn from_queries(source: &str, language: &'a Language) -> BTreeMap<String, Self> {
        let parsed = ts_query::Query::parse(source).unwrap();
        let state = Arc::new(State::new(language));
        let mut queries = BTreeMap::new();
        for node in parsed.children() {
            match node {
                ts_query::ProgramChildren::NamedNode(named) => {
                    let query = Self::from_named_node(&named, language, state.clone());
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
    fn from_named_node(
        named: &ts_query::NamedNode,
        language: &'a Language,
        state: Arc<State<'a>>,
    ) -> Self {
        Query {
            node: named.clone(),
            language: language,
            state: state,
        }
    }
    /// Get the kind of the query (the node to be matched)
    pub fn kind(&self) -> String {
        if let ts_query::NamedNodeName::Identifier(identifier) = &(*self.node.name) {
            return identifier.source();
        }
        panic!("No kind found for query. {:#?}", self.node);
    }
    pub fn struct_name(&self) -> String {
        normalize_type_name(&self.kind(), true)
    }
    pub fn struct_variants(&self) -> Vec<String> {
        self.state
            .get_variants(&self.struct_name())
            .iter()
            .map(|v| v.normalize())
            .filter(|v| v != "Comment")
            .collect()
    }

    fn captures(&self) -> Vec<ts_query::Capture> {
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
    // fn execute<T: HasChildren>(&self, node: &T) -> Vec<Box<dyn CSTNode + Send>> {
    //     let mut result = Vec::new();

    //     for child in node.children() {
    //         if self
    //             .captures()
    //             .iter()
    //             .any(|capture| capture.source() == child.kind())
    //         {
    //             result.push(child);
    //         }
    //     }
    //     result
    // }
    pub fn node(&self) -> &ts_query::NamedNode {
        &self.node
    }
    pub fn executor_id(&self) -> Ident {
        format_ident!("{}s", self.kind())
    }
    fn get_field_for_field_name(&self, field_name: &str, struct_name: &str) -> Option<&Field> {
        println!(
            "Getting field for: {:#?} on node: {:#?}",
            field_name, struct_name
        );
        let node = self.state.get_node_for_struct_name(struct_name);
        if let Some(node) = node {
            return node.get_field_for_field_name(field_name);
        }
        println!(
            "No node found for: {:#?}. In language: {:#?}",
            struct_name,
            self.language.name()
        );
        None
    }
    fn get_matcher_for_field(
        &self,
        field: &ts_query::FieldDefinition,
        struct_name: &str,
        current_node: &Ident,
    ) -> TokenStream {
        let other_child: ts_query::NodeTypes = field.children().pop().unwrap().into();
        for name in &field.name {
            if let ts_query::FieldDefinitionName::Identifier(identifier) = name {
                let name = identifier.source();
                if let Some(field) = self.get_field_for_field_name(&name, struct_name) {
                    let field_name = format_ident!("{}", name);
                    let new_identifier = format_ident!("field");
                    let normalized_struct_name = field.type_name();
                    let wrapped = self.get_matcher_for_definition(
                        &normalized_struct_name,
                        other_child,
                        &new_identifier,
                    );
                    if !field.is_optional() {
                        return quote! {
                            let #new_identifier = #current_node.#field_name;
                            #wrapped
                        };
                    } else {
                        return quote! {
                            if let Some(field) = #current_node.#field_name {
                                #wrapped
                            }
                        };
                    }
                }
            }
        }
        panic!(
            "No field found for: {:#?}. In language: {:#?}",
            field.source(),
            self.language.name()
        )
    }
    fn get_matchers_for_grouping(
        &self,
        node: &ts_query::Grouping,
        struct_name: &str,
        current_node: &Ident,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        for group in self.node().children() {
            let result = self.get_matcher_for_definition(struct_name, group.into(), current_node);
            matchers.extend_one(result);
        }
        matchers
    }

    fn get_matcher_for_named_node(
        &self,
        node: &ts_query::NamedNode,
        struct_name: &str,
        current_node: &Ident,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        let name = node.children().first().unwrap().source();
        let normalized_name = normalize_type_name(&name, true);
        for child in node.children().into_iter().skip(1) {
            let result =
                self.get_matcher_for_definition(&normalized_name, child.into(), current_node);
            matchers.extend_one(result);
        }
        matchers
    }
    fn get_matcher_for_definition(
        &self,
        struct_name: &str,
        node: ts_query::NodeTypes,
        current_node: &Ident,
    ) -> TokenStream {
        match node {
            ts_query::NodeTypes::FieldDefinition(field) => {
                self.get_matcher_for_field(&field, struct_name, current_node)
            }
            ts_query::NodeTypes::Capture(named) => {
                info!("Capture: {:#?}", named.source());
                quote! {}
            }
            ts_query::NodeTypes::NamedNode(named) => {
                self.get_matcher_for_named_node(&named, struct_name, current_node)
            }
            ts_query::NodeTypes::Comment(_) => {
                quote! {}
            }
            ts_query::NodeTypes::Grouping(grouping) => {
                self.get_matchers_for_grouping(&grouping, struct_name, current_node)
            }
            ts_query::NodeTypes::Identifier(identifier) => {
                let to_append = self.executor_id();
                let language = format_ident!("{}", self.language.name());
                let children: Ident = format_ident!("{}Children", struct_name);
                let struct_name =
                    format_ident!("{}", normalize_type_name(&identifier.source(), true));
                quote! {
                    if #current_node.children().any(|child| {
                        if let #language::#children::#struct_name(_) = child {
                            true
                        } else {
                            false
                        }
                    }) {
                        #to_append.push(node);
                    }
                }
            }
            unhandled => todo!(
                "Unhandled definition in language {}: {:#?}, {:#?}",
                self.language.name(),
                unhandled.kind(),
                unhandled.source()
            ),
        }
    }

    pub fn matcher(&self, struct_name: &str) -> TokenStream {
        info!(
            "Generating matcher for: {:#?}. Has children: {:#?}",
            self.node().source(),
            self.node().children()
        );
        let mut matchers = TokenStream::new();
        for child in self.node().children().into_iter().skip(1) {
            let result =
                self.get_matcher_for_definition(struct_name, child.into(), &format_ident!("node"));
            matchers.extend_one(result);
        }
        matchers
    }
}

pub trait HasQuery {
    fn queries(&self) -> BTreeMap<String, Query>;
    fn queries_with_prefix(&self, prefix: &str) -> BTreeMap<String, Query> {
        let mut queries = BTreeMap::new();
        for (name, query) in self.queries().into_iter() {
            if name.starts_with(prefix) {
                let new_name = name.split(".").last().unwrap();
                queries.insert(new_name.to_string(), query);
            }
        }
        queries
    }
    fn definitions(&self) -> BTreeMap<String, Query<'_>> {
        self.queries_with_prefix("@definition")
    }
    fn references(&self) -> BTreeMap<String, Query<'_>> {
        self.queries_with_prefix("@reference")
    }
}
impl HasQuery for Language {
    fn queries(&self) -> BTreeMap<String, Query<'_>> {
        Query::from_queries(&self.tag_query, self)
    }
}
