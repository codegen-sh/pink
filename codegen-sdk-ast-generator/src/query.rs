use std::{collections::BTreeMap, sync::Arc};

use codegen_sdk_common::{
    CSTNode, HasChildren, Language,
    naming::{normalize_field_name, normalize_type_name},
};
use codegen_sdk_cst::{CSTLanguage, ts_query};
use codegen_sdk_cst_generator::{Field, State};
use derive_more::Debug;
use log::{debug, info, warn};
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
        if self
            .state
            .get_node_for_struct_name(&self.struct_name())
            .is_some()
        {
            return vec![self.struct_name()];
        }
        self.state
            .get_variants(&self.struct_name())
            .into_iter()
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
        let raw_name = self.name();
        let name = raw_name.split(".").last().unwrap();
        if name.ends_with("s") {
            format_ident!("{}es", name)
        } else {
            format_ident!("{}s", name)
        }
    }
    fn get_field_for_field_name(&self, field_name: &str, struct_name: &str) -> Option<&Field> {
        debug!(
            "Getting field for: {:#?} on node: {:#?}",
            field_name, struct_name
        );
        let node = self.state.get_node_for_struct_name(struct_name);
        if let Some(node) = node {
            return node.get_field_for_field_name(field_name);
        }
        warn!(
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
        let other_child: ts_query::NodeTypes =
            field.children().into_iter().skip(2).next().unwrap().into();
        for name in &field.name {
            if let ts_query::FieldDefinitionName::Identifier(identifier) = name {
                let name = normalize_field_name(&identifier.source());
                if let Some(field) = self.get_field_for_field_name(&name, struct_name) {
                    let field_name = format_ident!("{}", name);
                    let new_identifier = format_ident!("field");
                    let normalized_struct_name = field.type_name();
                    let wrapped = self.get_matcher_for_definition(
                        &normalized_struct_name,
                        other_child.clone(),
                        &new_identifier,
                    );
                    assert!(
                        wrapped.to_string().len() > 0,
                        "Wrapped is empty, {} {} {}",
                        normalized_struct_name,
                        other_child.source(),
                        other_child.kind()
                    );
                    if !field.is_optional() {
                        return quote! {
                            let #new_identifier = &#current_node.#field_name;
                            #wrapped
                        };
                    } else {
                        return quote! {
                            if let Some(field) = &#current_node.#field_name {
                                #wrapped
                            }
                        };
                    }
                } else {
                    panic!(
                        "No field found for: {:#?} on node: {:#?}. In language: {:#?}. Field source: {:#?}",
                        name,
                        struct_name,
                        self.language.name(),
                        field.source()
                    )
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
        for group in node.children() {
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
        let first_node = node.children().into_iter().next().unwrap();
        let remaining_nodes = node
            .children()
            .into_iter()
            .skip(1)
            .filter(|child| child.kind() != "capture")
            .collect::<Vec<_>>();
        if remaining_nodes.len() == 0 {
            log::info!("single node, {}", first_node.source());
            return self.get_matcher_for_definition(struct_name, first_node.into(), current_node);
        }

        let name_node = self.state.get_node_for_raw_name(&first_node.source());
        if let Some(name_node) = name_node {
            for child in remaining_nodes {
                let result = self.get_matcher_for_definition(
                    &name_node.normalize_name(),
                    child.into(),
                    current_node,
                );
                matchers.extend_one(result);
            }
        } else {
            let subenum = self.state.get_subenum_variants(&first_node.source());
            log::info!(
                "subenum {} with {} variants",
                first_node.source(),
                subenum.len()
            );
            for variant in subenum {
                if variant.normalize_name() == "Comment" {
                    continue;
                }
                for child in remaining_nodes.clone() {
                    let result = self.get_matcher_for_definition(
                        &variant.normalize_name(),
                        child.into(),
                        current_node,
                    );
                    matchers.extend_one(result);
                }
            }
        }
        matchers
    }
    fn get_default_matcher(&self) -> TokenStream {
        let to_append = self.executor_id();
        quote! {
            self.#to_append.push(node.clone());
        }
    }
    fn get_matcher_for_definition(
        &self,
        struct_name: &str,
        node: ts_query::NodeTypes,
        current_node: &Ident,
    ) -> TokenStream {
        if !node.is_named() {
            return self.get_default_matcher();
        }
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
            ts_query::NodeTypes::List(subenum) => {
                for child in subenum.children() {
                    let result =
                        self.get_matcher_for_definition(struct_name, child.into(), current_node);
                    // Currently just returns the first child
                    return result; // TODO: properly handle list
                }
                quote! {}
            }
            ts_query::NodeTypes::Grouping(grouping) => {
                self.get_matchers_for_grouping(&grouping, struct_name, current_node)
            }
            ts_query::NodeTypes::Identifier(identifier) => {
                let to_append = self.get_default_matcher();
                let language = format_ident!("{}", self.language.name());
                let children;
                if let Some(node) = self.state.get_node_for_struct_name(struct_name) {
                    children = format_ident!("{}Children", struct_name);
                    // When there is only 1 possible child, we can use the default matcher
                    if node.children_struct_name() != children.to_string() {
                        return self.get_default_matcher();
                    }
                } else {
                    // If this is a field, we may be dealing with multiple types and can't operate over all of them
                    return self.get_default_matcher(); // TODO: Handle this case
                }
                let struct_name =
                    format_ident!("{}", normalize_type_name(&identifier.source(), true));
                quote! {
                    if #current_node.children().into_iter().any(|child| {
                        if let #language::#children::#struct_name(_) = child {
                            true
                        } else {
                            false
                        }
                    }) {
                        #to_append
                    }
                }
            }
            unhandled => {
                log::warn!(
                    "Unhandled definition in language {}: {:#?}, {:#?}",
                    self.language.name(),
                    unhandled.kind(),
                    unhandled.source()
                );
                self.get_default_matcher()
            }
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
    fn queries_with_prefix(&self, prefix: &str) -> BTreeMap<String, Vec<Query<'_>>> {
        let mut queries = BTreeMap::new();
        for (name, query) in self.queries().into_iter() {
            if name.starts_with(prefix) {
                let new_name = name.split(".").last().unwrap();
                queries
                    .entry(new_name.to_string())
                    .or_insert(Vec::new())
                    .push(query);
            }
        }
        queries
    }
}
impl HasQuery for Language {
    fn queries(&self) -> BTreeMap<String, Query<'_>> {
        Query::from_queries(&self.tag_query, self)
    }
}
