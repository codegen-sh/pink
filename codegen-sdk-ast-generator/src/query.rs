use std::{collections::BTreeMap, sync::Arc};

use codegen_sdk_common::{
    CSTNode, HasChildren, Language, Tree,
    naming::{normalize_field_name, normalize_type_name},
};
use codegen_sdk_cst::CSTLanguage;
use codegen_sdk_cst_generator::{Config, Field, State};
use codegen_sdk_ts_query::cst as ts_query;
use derive_more::Debug;
use log::{debug, info, warn};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use ts_query::NodeTypes;
fn captures_for_field_definition<'a>(
    node: &'a ts_query::FieldDefinition<'a>,
    tree: &'a Tree<NodeTypes<'a>>,
) -> impl Iterator<Item = &'a ts_query::Capture<'a>> {
    let mut captures = Vec::new();
    for child in node.children(tree) {
        match child {
            ts_query::FieldDefinitionChildrenRef::NamedNode(named) => {
                captures.extend(captures_for_named_node(&named, tree));
            }
            ts_query::FieldDefinitionChildrenRef::FieldDefinition(field) => {
                captures.extend(captures_for_field_definition(&field, tree));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
fn captures_for_named_node<'a>(
    node: &'a ts_query::NamedNode<'a>,
    tree: &'a Tree<NodeTypes<'a>>,
) -> impl Iterator<Item = &'a ts_query::Capture<'a>> {
    let mut captures = Vec::new();
    for child in node.children(tree) {
        match child {
            ts_query::NamedNodeChildrenRef::Capture(capture) => captures.push(capture),
            ts_query::NamedNodeChildrenRef::NamedNode(named) => {
                captures.extend(captures_for_named_node(&named, tree));
            }
            ts_query::NamedNodeChildrenRef::FieldDefinition(field) => {
                captures.extend(captures_for_field_definition(&field, tree));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
#[derive(Debug)]
pub struct Query<'a> {
    node: &'a ts_query::NamedNode<'a>,
    language: &'a Language,
    tree: &'a Tree<NodeTypes<'a>>,
    pub(crate) state: Arc<State<'a>>,
}
impl<'a> Query<'a> {
    pub fn from_queries(
        db: &'a dyn salsa::Database,
        source: &str,
        language: &'a Language,
    ) -> BTreeMap<String, Self> {
        let result = ts_query::Query::parse(db, source.to_string()).unwrap();
        let (parsed, tree) = result;
        let config = Config::default();
        let state = Arc::new(State::new(language, config));
        let mut queries = BTreeMap::new();
        for node in parsed.children(tree) {
            match node {
                ts_query::ProgramChildrenRef::NamedNode(named) => {
                    let query = Self::from_named_node(&named, language, state.clone(), tree);
                    queries.insert(query.name(), query);
                }
                node => {
                    log::warn!(
                        "Unhandled query: {:#?}. Source: {:#?}",
                        node.kind_name(),
                        node.source()
                    );
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
        named: &'a ts_query::NamedNode<'a>,
        language: &'a Language,
        state: Arc<State<'a>>,
        tree: &'a Tree<NodeTypes<'a>>,
    ) -> Self {
        Query {
            node: named,
            language: language,
            state: state,
            tree: tree,
        }
    }
    /// Get the kind of the query (the node to be matched)
    pub fn kind(&self) -> String {
        if let ts_query::NamedNodeNameRef::Identifier(identifier) = self.node.name(self.tree) {
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
            .get_variants(&self.struct_name(), false)
            .into_iter()
            .map(|v| v.normalize())
            .filter(|v| v != "Comment")
            .collect()
    }

    fn captures(&self) -> Vec<&ts_query::Capture> {
        captures_for_named_node(&self.node, self.tree).collect()
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
        name_value: Option<TokenStream>,
    ) -> TokenStream {
        let other_child: ts_query::NodeTypesRef = field
            .children(self.tree)
            .into_iter()
            .skip(2)
            .next()
            .unwrap()
            .into();
        for name in &field.name(self.tree) {
            if let ts_query::FieldDefinitionNameRef::Identifier(identifier) = name {
                let name = normalize_field_name(&identifier.source());
                if let Some(field) = self.get_field_for_field_name(&name, struct_name) {
                    let field_name = format_ident!("{}", name);
                    let normalized_struct_name = field.type_name();
                    let wrapped = self.get_matcher_for_definition(
                        &normalized_struct_name,
                        other_child.clone(),
                        &field_name,
                        name_value,
                    );
                    // assert!(
                    //     wrapped.to_string().len() > 0,
                    //     "Wrapped is empty, {} {} {}",
                    //     normalized_struct_name,
                    //     other_child.source(),
                    //     other_child.kind()
                    // );
                    if !field.is_optional() {
                        return quote! {
                            let #field_name = #current_node.#field_name(tree);
                            #wrapped
                        };
                    } else {
                        return quote! {
                            if let Some(#field_name) = #current_node.#field_name(tree) {
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
        name_value: Option<TokenStream>,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        for group in node.children(self.tree) {
            let result = self.get_matcher_for_definition(
                struct_name,
                group.into(),
                current_node,
                name_value.clone(),
            );
            matchers.extend_one(result);
        }
        matchers
    }
    fn _get_matcher_for_named_node(
        &self,
        struct_name: &str,
        target_name: &str,
        target_kind: &str,
        current_node: &Ident,
        remaining_nodes: Vec<ts_query::NamedNodeChildrenRef<'_>>,
        name_value: Option<TokenStream>,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        let mut field_matchers = TokenStream::new();
        let mut comment_variant = None;
        let variants = self
            .state
            .get_variants(&format!("{}Children", target_kind), true);
        if variants.len() == 2 {
            if variants.iter().any(|v| v.normalize() == "Comment") {
                for variant in variants {
                    if variant.normalize() == "Comment" {
                        continue;
                    }
                    comment_variant = Some(variant.normalize());
                }
            }
        }

        for child in remaining_nodes {
            if child.kind_name() == "field_definition" {
                field_matchers.extend_one(self.get_matcher_for_definition(
                    &target_name,
                    child.into(),
                    current_node,
                    name_value.clone(),
                ));
            } else {
                let result = self.get_matcher_for_definition(
                    &target_name,
                    child.into(),
                    &format_ident!("child"),
                    name_value.clone(),
                );

                if let Some(ref variant) = comment_variant {
                    let children = format_ident!("{}Children", target_name);
                    let variant = format_ident!("{}Ref", variant);
                    matchers.extend_one(quote! {
                        if let crate::cst::#children::#variant(#current_node) = #current_node {
                            #result
                        }
                    });
                } else {
                    matchers.extend_one(quote! {
                        #result
                    });
                }
            }
        }
        let matchers = if matchers.is_empty() {
            quote! {}
        } else {
            quote! {
                for child in #current_node.children(tree) {
                    #matchers
                    break;
                }
            }
        };
        let query_source = format!(
            "Code for query: {}",
            &self.node().source().replace("\n", " ") // Newlines mess with quote's doc comments
        );
        if matchers.is_empty() && field_matchers.is_empty() {
            return quote! {};
        }
        let base_matcher = quote! {
            #[doc = #query_source]
            #matchers
            #field_matchers
        };
        if struct_name == target_name {
            return base_matcher;
        } else {
            let mut children = format_ident!("{}Ref", struct_name);
            if let Some(node) = self.state.get_node_for_struct_name(struct_name) {
                children = format_ident!("{}Ref", node.children_struct_name());
            }
            let variant = format_ident!("{}", target_name);
            return quote! {
                if let crate::cst::#children::#variant(#current_node) = #current_node {
                    #base_matcher
                }
            };
        }
    }
    fn group_children<'b>(
        &'b self,
        node: &'b ts_query::NamedNode<'b>,
        first_node: &ts_query::NamedNodeChildrenRef<'_>,
        mut name_value: Option<TokenStream>,
        current_node: &Ident,
    ) -> (Option<TokenStream>, Vec<ts_query::NamedNodeChildrenRef<'b>>) {
        let mut prev = first_node.clone();
        let mut remaining_nodes = Vec::new();
        for child in node.children(self.tree).into_iter().skip(1) {
            if child.kind_name() == "capture" {
                if child.source() == "@name" {
                    log::info!(
                        "Found @name! prev: {:#?}, {:#?}",
                        prev.source(),
                        prev.kind_name()
                    );
                    match prev {
                        ts_query::NamedNodeChildrenRef::FieldDefinition(field) => {
                            let field_name = field
                                .name(self.tree)
                                .iter()
                                .filter(|c| c.is_named())
                                .map(|c| format_ident!("{}", c.source()))
                                .next()
                                .unwrap();
                            name_value = Some(quote! {
                                #current_node.#field_name.source()
                            });
                        }
                        ts_query::NamedNodeChildrenRef::Identifier(named) => {
                            log::info!(
                                "Found @name! prev: {:#?}, {:#?}",
                                named.source(),
                                named.kind_name()
                            );
                            name_value = Some(quote! {
                                #current_node.source()
                            });
                        }
                        ts_query::NamedNodeChildrenRef::AnonymousUnderscore(_) => {
                            name_value = Some(quote! {
                                #current_node.source()
                            });
                        }
                        _ => panic!(
                            "Unexpected prev: {:#?}, source: {:#?}. Query: {:#?}",
                            prev.kind_name(),
                            prev.source(),
                            self.node().source()
                        ),
                    }
                    break;
                }
                continue;
            }
            prev = child.clone();
            remaining_nodes.push(child);
        }
        (name_value, remaining_nodes)
    }
    fn get_matcher_for_named_node(
        &self,
        node: &ts_query::NamedNode,
        struct_name: &str,
        current_node: &Ident,
        name_value: Option<TokenStream>,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        let first_node = node.children(self.tree).into_iter().next().unwrap();
        let (name_value, remaining_nodes) =
            self.group_children(node, &first_node, name_value, current_node);
        if remaining_nodes.len() == 0 {
            log::info!("single node, {}", first_node.source());
            return self.get_matcher_for_definition(
                struct_name,
                first_node.into(),
                current_node,
                name_value,
            );
        }

        let name_node = self.state.get_node_for_raw_name(&first_node.source());
        if let Some(name_node) = name_node {
            let target_name = name_node.normalize_name();
            let matcher = self._get_matcher_for_named_node(
                struct_name,
                &target_name,
                name_node.kind(),
                current_node,
                remaining_nodes,
                name_value,
            );
            matchers.extend_one(matcher);
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
                let matcher = self._get_matcher_for_named_node(
                    struct_name,
                    &variant.normalize_name(),
                    variant.kind(),
                    current_node,
                    remaining_nodes.clone(),
                    name_value.clone(),
                );
                matchers.extend_one(matcher);
            }
        }
        quote! {
            #matchers
        }
    }
    fn get_default_matcher(&self, name_value: Option<TokenStream>) -> TokenStream {
        let to_append = self.executor_id();
        if let Some(name_value) = name_value {
            return quote! {
                #to_append.entry(#name_value).or_default().push(id);
            };
        }
        log::warn!("No name value found for: {}", self.node().source());
        quote! {}
    }
    fn get_matcher_for_identifier(
        &self,
        identifier: &ts_query::Identifier,
        struct_name: &str,
        current_node: &Ident,
        name_value: Option<TokenStream>,
    ) -> TokenStream {
        // We have 2 nodes, the parent node and the identifier node
        let to_append = self.get_default_matcher(name_value);
        // Case 1: The identifier is the same as the struct name (IE: we know this is the corrent node)
        if normalize_type_name(&identifier.source(), true) == struct_name {
            return to_append;
        }
        // Case 2: We have a node for the parent struct
        if let Some(node) = self.state.get_node_for_struct_name(struct_name) {
            let mut children = format_ident!("{}Children", struct_name);
            // When there is only 1 possible child, we can use the default matcher
            if node.children_struct_name() != children.to_string() {
                return to_append;
            }
            children = format_ident!("{}ChildrenRef", struct_name);
            let struct_name = format_ident!("{}", normalize_type_name(&identifier.source(), true));
            quote! {
                if let crate::cst::#children::#struct_name(child) = #current_node {
                    #to_append
                }

            }
        } else {
            // Case 3: This is a subenum
            // If this is a field, we may be dealing with multiple types and can't operate over all of them
            return to_append; // TODO: Handle this case
        }
    }
    fn get_matcher_for_definition(
        &self,
        struct_name: &str,
        node: ts_query::NodeTypesRef,
        current_node: &Ident,
        name_value: Option<TokenStream>,
    ) -> TokenStream {
        if !node.is_named() {
            return self.get_default_matcher(name_value);
        }
        match node {
            ts_query::NodeTypesRef::FieldDefinition(field) => {
                self.get_matcher_for_field(&field, struct_name, current_node, name_value)
            }
            ts_query::NodeTypesRef::Capture(named) => {
                info!("Capture: {:#?}", named.source());
                quote! {}
            }
            ts_query::NodeTypesRef::NamedNode(named) => {
                self.get_matcher_for_named_node(&named, struct_name, current_node, name_value)
            }
            ts_query::NodeTypesRef::Comment(_) => {
                quote! {}
            }
            ts_query::NodeTypesRef::List(subenum) => {
                for child in subenum.children(self.tree) {
                    let result = self.get_matcher_for_definition(
                        struct_name,
                        child.into(),
                        current_node,
                        name_value.clone(),
                    );
                    // Currently just returns the first child
                    return result; // TODO: properly handle list
                }
                quote! {}
            }
            ts_query::NodeTypesRef::Grouping(grouping) => {
                self.get_matchers_for_grouping(&grouping, struct_name, current_node, name_value)
            }
            ts_query::NodeTypesRef::Identifier(identifier) => {
                self.get_matcher_for_identifier(&identifier, struct_name, current_node, name_value)
            }
            unhandled => {
                log::warn!(
                    "Unhandled definition in language {}: {:#?}, {:#?}",
                    self.language.name(),
                    unhandled.kind_name(),
                    unhandled.source()
                );
                self.get_default_matcher(name_value)
            }
        }
    }

    pub fn matcher(&self, struct_name: &str) -> TokenStream {
        let node = self.state.get_node_for_struct_name(struct_name);
        let kind = if let Some(node) = node {
            node.kind()
        } else {
            struct_name
        };
        let starting_node = format_ident!("node");
        let (name_value, remaining_nodes) = self.group_children(
            &self.node(),
            &self.node().children(self.tree).into_iter().next().unwrap(),
            None,
            &starting_node,
        );
        return self._get_matcher_for_named_node(
            struct_name,
            &struct_name,
            kind,
            &starting_node,
            remaining_nodes,
            name_value,
        );
    }
}

pub trait HasQuery {
    fn queries<'a, 'db: 'a>(&'a self, db: &'db dyn salsa::Database) -> BTreeMap<String, Query<'a>>;
    fn queries_with_prefix<'a, 'db: 'a>(
        &'a self,
        db: &'db dyn salsa::Database,
        prefix: &str,
    ) -> BTreeMap<String, Vec<Query<'a>>> {
        let mut queries = BTreeMap::new();
        for (name, query) in self.queries(db).into_iter() {
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
    fn queries<'a, 'db: 'a>(&'a self, db: &'db dyn salsa::Database) -> BTreeMap<String, Query<'a>> {
        Query::from_queries(db, &self.tag_query, self)
    }
}
#[cfg(test)]
mod tests {
    use codegen_sdk_common::language::ts_query;
    use codegen_sdk_cst::CSTDatabase;

    use super::*;
    #[test]
    fn test_query_basic() {
        let database = CSTDatabase::default();
        let language = &ts_query::Query;
        let queries = Query::from_queries(&database, "(abc) @definition.abc", language);
        assert!(queries.len() > 0);
    }
}
