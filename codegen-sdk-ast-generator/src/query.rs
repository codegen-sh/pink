use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use codegen_sdk_common::{
    CSTNode, HasChildren, Language, Tree,
    naming::{normalize_field_name, normalize_type_name},
};
use codegen_sdk_cst::CSTLanguage;
use codegen_sdk_cst_generator::{Config, Field, State};
use codegen_sdk_ts_query::cst as ts_query;
use derive_more::Debug;
use indextree::NodeId;
use log::{debug, info, warn};
pub mod field;
use anyhow::Error;
pub mod symbol;
use pluralizer::pluralize;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use ts_query::NodeTypes;
fn name_for_capture<'a>(capture: &'a ts_query::Capture<'a>) -> String {
    full_name_for_capture(capture)
        .split(".")
        .last()
        .unwrap()
        .to_string()
}
fn full_name_for_capture<'a>(capture: &'a ts_query::Capture<'a>) -> String {
    capture.source().split_off(1)
}
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
            ts_query::FieldDefinitionChildrenRef::Grouping(grouping) => {
                captures.extend(captures_for_grouping(&grouping, tree));
            }
            ts_query::FieldDefinitionChildrenRef::List(list) => {
                captures.extend(captures_for_list(&list, tree));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
fn captures_for_list<'a>(
    list: &'a ts_query::List<'a>,
    tree: &'a Tree<NodeTypes<'a>>,
) -> impl Iterator<Item = &'a ts_query::Capture<'a>> {
    let mut captures = Vec::new();
    for child in list.children(tree) {
        match child {
            ts_query::ListChildrenRef::NamedNode(named) => {
                captures.extend(captures_for_named_node(&named, tree));
            }
            ts_query::ListChildrenRef::List(list) => {
                captures.extend(captures_for_list(&list, tree));
            }
            ts_query::ListChildrenRef::FieldDefinition(field) => {
                captures.extend(captures_for_field_definition(&field, tree));
            }
            _ => {}
        }
    }
    captures.into_iter()
}
fn captures_for_grouping<'a>(
    grouping: &'a ts_query::Grouping<'a>,
    tree: &'a Tree<NodeTypes<'a>>,
) -> impl Iterator<Item = &'a ts_query::Capture<'a>> {
    let mut captures = Vec::new();
    for child in grouping.children(tree) {
        match child {
            ts_query::GroupingChildrenRef::NamedNode(named) => {
                captures.extend(captures_for_named_node(&named, tree));
            }
            ts_query::GroupingChildrenRef::Grouping(grouping) => {
                captures.extend(captures_for_grouping(&grouping, tree));
            }
            ts_query::GroupingChildrenRef::FieldDefinition(field) => {
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
            ts_query::NamedNodeChildrenRef::Grouping(grouping) => {
                captures.extend(captures_for_grouping(&grouping, tree));
            }
            ts_query::NamedNodeChildrenRef::List(list) => {
                captures.extend(captures_for_list(&list, tree));
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
    root_id: NodeId,
    pub(crate) state: Arc<State<'a>>,
}
impl<'a> Query<'a> {
    pub fn from_queries(
        db: &'a dyn salsa::Database,
        source: &str,
        language: &'a Language,
    ) -> BTreeMap<String, Self> {
        let result = ts_query::Query::parse(db, source.to_string()).unwrap();
        let (parsed, tree, program_id) = result;
        let config = Config::default();
        let state = Arc::new(State::new(language, config));
        let mut queries = BTreeMap::new();
        for node in parsed.children(tree) {
            match node {
                ts_query::ProgramChildrenRef::NamedNode(named) => {
                    let query =
                        Self::from_named_node(&named, language, state.clone(), tree, program_id);
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
        root_id: indextree::NodeId,
    ) -> Self {
        Query {
            node: named,
            language: language,
            state: state,
            tree: tree,
            root_id: root_id,
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
        let pluralized = pluralize(name, 2, false);
        format_ident!("{}", pluralized)
    }
    pub fn symbol_name(&self) -> Ident {
        let raw_name = self.name();
        let name = raw_name.split(".").last().unwrap();
        let symbol = format_ident!("{}", normalize_type_name(name, true));
        // References can produce duplicate names. We can be reasonably sure that there is no @definition.call.
        if raw_name.starts_with("reference") && !["call"].contains(&name) {
            format_ident!("{}Ref", symbol)
        } else {
            symbol
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
        _current_node_id: &Ident,
        existing: &mut Vec<(ts_query::NodeTypesRef, &str, &Ident, &Ident)>,
        query_values: &mut HashMap<String, TokenStream>,
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
                let doc = format!("Code for field: {}", field.source());
                let name = normalize_field_name(&identifier.source());
                if let Some(field) = self.get_field_for_field_name(&name, struct_name) {
                    let field_name = format_ident!("{}", name);
                    let field_name_id = format_ident!("{}_id", name);

                    let normalized_struct_name = field.type_name();
                    let wrapped = self.get_matcher_for_definition(
                        &normalized_struct_name,
                        other_child.clone(),
                        &field_name,
                        &field_name_id,
                        existing,
                        query_values,
                    );
                    // assert!(
                    //     wrapped.to_string().len() > 0,
                    //     "Wrapped is empty, {} {} {}",
                    //     normalized_struct_name,
                    //     other_child.source(),
                    //     other_child.kind()
                    // );
                    if field.is_multiple() {
                        return quote! {
                            #[doc = #doc]
                            for (#field_name, #field_name_id) in #current_node.#field_name(tree).iter().zip(#current_node.#field_name.iter()) {
                                #wrapped
                            }
                        };
                    } else if !field.is_optional() {
                        return quote! {
                            #[doc = #doc]
                            let #field_name_id = #current_node.#field_name;
                            let #field_name = #current_node.#field_name(tree);
                            #wrapped
                        };
                    } else {
                        return quote! {
                            #[doc = #doc]
                            if let Some(#field_name) = #current_node.#field_name(tree) {
                                let #field_name_id = #current_node.#field_name.unwrap();
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
        current_node_id: &Ident,
        existing: &mut Vec<(ts_query::NodeTypesRef, &str, &Ident, &Ident)>,
        query_values: &mut HashMap<String, TokenStream>,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        for group in node.children(self.tree) {
            let result = self.get_matcher_for_definition(
                struct_name,
                group.into(),
                current_node,
                current_node_id,
                existing,
                query_values,
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
        current_node_id: &Ident,
        remaining_nodes: Vec<ts_query::NamedNodeChildrenRef<'_>>,
        query_values: &mut HashMap<String, TokenStream>,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        let mut field_matchers = Vec::new();
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
            if let ts_query::NamedNodeChildrenRef::FieldDefinition(_) = child {
                field_matchers.push((child.into(), target_name, current_node, current_node_id));
            } else {
                let result = self.get_matcher_for_definition(
                    &target_name,
                    child.into(),
                    &format_ident!("child"),
                    current_node_id,
                    &mut Vec::new(),
                    query_values,
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
        let field_matchers = if let Some(prev) = field_matchers.pop() {
            self.get_matcher_for_definition(
                &prev.1,
                prev.0,
                &prev.2,
                &prev.3,
                &mut field_matchers,
                query_values,
            )
        } else {
            quote! {}
        };
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
        query_values: &mut HashMap<String, TokenStream>,
        current_node: &Ident,
        current_node_id: &Ident,
    ) -> Vec<ts_query::NamedNodeChildrenRef<'b>> {
        let mut prev = first_node.clone();
        let mut remaining_nodes = Vec::new();
        log::info!(
            "Grouping children for: {:#?} of kind: {:#?}",
            node.source(),
            node.kind_name()
        );
        for child in node.children(self.tree).into_iter().skip(1) {
            if child.kind_name() == "capture" {
                let capture_name = name_for_capture(child.try_into().unwrap());
                if self.target_capture_names().contains(&capture_name) {
                    match prev {
                        ts_query::NamedNodeChildrenRef::FieldDefinition(field) => {
                            log::info!("Found @{}! on field: {:#?}", capture_name, field.source(),);
                            let field_name = field
                                .name(self.tree)
                                .iter()
                                .filter(|c| c.is_named())
                                .map(|c| format_ident!("{}", c.source()))
                                .next()
                                .unwrap();
                            let msg =
                                format!("Found @{}! on field: {:#?}", capture_name, field.source());
                            query_values.insert(
                                capture_name,
                                quote! {
                                    #[doc = #msg]
                                    #current_node.#field_name
                                },
                            );
                        }
                        ts_query::NamedNodeChildrenRef::Identifier(named) => {
                            log::info!(
                                "Found @{}! prev: {:#?}, {:#?}",
                                capture_name,
                                named.source(),
                                named.kind_name()
                            );
                            query_values.insert(
                                capture_name,
                                quote! {
                                    #current_node_id
                                },
                            );
                        }
                        ts_query::NamedNodeChildrenRef::AnonymousUnderscore(_) => {
                            log::info!(
                                "Found @{}! on anonymous underscore: {:#?}",
                                capture_name,
                                node.source()
                            );
                            query_values.insert(
                                capture_name,
                                quote! {
                                    #current_node_id
                                },
                            );
                        }
                        _ => panic!(
                            "Unexpected prev: {:#?}, source: {:#?}. Query: {:#?}",
                            prev.kind_name(),
                            prev.source(),
                            self.node().source()
                        ),
                    }
                }
                continue;
            }
            prev = child.clone();
            remaining_nodes.push(child);
        }
        remaining_nodes
    }
    fn get_matcher_for_named_node(
        &self,
        node: &ts_query::NamedNode,
        struct_name: &str,
        current_node: &Ident,
        current_node_id: &Ident,
        existing: &mut Vec<(ts_query::NodeTypesRef, &str, &Ident, &Ident)>,
        query_values: &mut HashMap<String, TokenStream>,
    ) -> TokenStream {
        let mut matchers = TokenStream::new();
        let first_node = node.children(self.tree).into_iter().next().unwrap();
        let remaining_nodes = self.group_children(
            node,
            &first_node,
            query_values,
            current_node,
            current_node_id,
        );
        if remaining_nodes.len() == 0 {
            log::info!("single node, {}", first_node.source());
            return self.get_matcher_for_definition(
                struct_name,
                first_node.into(),
                current_node,
                current_node_id,
                existing,
                query_values,
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
                current_node_id,
                remaining_nodes,
                query_values,
            );
            matchers.extend_one(matcher);
        } else {
            let subenum = self.state.get_subenum_variants(&first_node.source(), false);
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
                    current_node_id,
                    remaining_nodes.clone(),
                    query_values,
                );
                matchers.extend_one(matcher);
            }
        }
        quote! {
            #matchers
        }
    }
    fn get_default_matcher(
        &self,
        existing: &mut Vec<(ts_query::NodeTypesRef, &str, &Ident, &Ident)>,
        query_values: &mut HashMap<String, TokenStream>,
    ) -> TokenStream {
        if let Some(prev) = existing.pop() {
            log::info!(
                "Executing previous matcher on: {:#?} with {:#?}",
                prev.0.source(),
                query_values
            );
            return self.get_matcher_for_definition(
                &prev.1,
                prev.0,
                &prev.2,
                &prev.3,
                existing,
                query_values,
            );
        }

        let to_append = self.executor_id();
        let mut args = Vec::new();
        for target in self.target_capture_names() {
            if let Some(value) = query_values.get(&target) {
                args.push(value);
            } else {
                log::warn!("No value found for: {} on {}", target, self.node().source());
                return quote! {};
            }
        }
        let name = query_values.get("name").unwrap_or_else(|| {
            panic!(
                "No name found for: {}. Query_values: {:#?} Target Capture Names: {:#?}",
                self.node().source(),
                query_values,
                self.target_capture_names()
            );
        });
        let symbol_name = self.symbol_name();
        return quote! {
            let name = tree.get(&#name).unwrap().source();
            let fully_qualified_name = codegen_sdk_resolution::FullyQualifiedName::new(db, node.file_id(),name.clone());
            let tree_id = codegen_sdk_common::CSTNodeTreeId::from_node_id(db, &node.id(), id);
            let symbol = #symbol_name::new(db, fully_qualified_name, tree_id, #(codegen_sdk_common::CSTNodeTreeId::from_node_id(db, &tree.get(&#args).unwrap().id(), #args.clone())),*);
            #to_append.entry(name).or_default().push(symbol);
        };
    }
    fn get_matcher_for_identifier(
        &self,
        identifier: &ts_query::Identifier,
        struct_name: &str,
        current_node: &Ident,
        _current_node_id: &Ident,
        existing: &mut Vec<(ts_query::NodeTypesRef, &str, &Ident, &Ident)>,
        query_values: &mut HashMap<String, TokenStream>,
    ) -> TokenStream {
        // We have 2 nodes, the parent node and the identifier node
        let to_append = self.get_default_matcher(existing, query_values);
        // Case 1: The identifier is the same as the struct name (IE: we know this is the corrent node)
        let target_name = normalize_type_name(&identifier.source(), true);
        if target_name == struct_name {
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
            let target_name = format_ident!("{}", target_name);
            let struct_name = format_ident!("{}Ref", struct_name);
            return quote! {
                if let crate::cst::#struct_name::#target_name(#current_node) = #current_node {
                    #to_append
                }
            }; // TODO: Handle this case
        }
    }
    fn get_matcher_for_definition(
        &self,
        struct_name: &str,
        node: ts_query::NodeTypesRef,
        current_node: &Ident,
        current_node_id: &Ident,
        existing: &mut Vec<(ts_query::NodeTypesRef, &str, &Ident, &Ident)>,
        query_values: &mut HashMap<String, TokenStream>,
    ) -> TokenStream {
        if !node.is_named() {
            return self.get_default_matcher(existing, query_values);
        }
        match node {
            ts_query::NodeTypesRef::FieldDefinition(field) => self.get_matcher_for_field(
                &field,
                struct_name,
                current_node,
                current_node_id,
                existing,
                query_values,
            ),
            ts_query::NodeTypesRef::Capture(named) => {
                info!("Capture: {:#?}", named.source());
                quote! {}
            }
            ts_query::NodeTypesRef::NamedNode(named) => self.get_matcher_for_named_node(
                &named,
                struct_name,
                current_node,
                current_node_id,
                existing,
                query_values,
            ),
            ts_query::NodeTypesRef::Comment(_) => {
                quote! {}
            }
            ts_query::NodeTypesRef::List(subenum) => {
                for child in subenum.children(self.tree) {
                    let result = self.get_matcher_for_definition(
                        struct_name,
                        child.into(),
                        current_node,
                        current_node_id,
                        existing,
                        query_values,
                    );
                    // Currently just returns the first child
                    return result; // TODO: properly handle list
                }
                quote! {}
            }
            ts_query::NodeTypesRef::Grouping(grouping) => self.get_matchers_for_grouping(
                &grouping,
                struct_name,
                current_node,
                current_node_id,
                existing,
                query_values,
            ),
            ts_query::NodeTypesRef::Identifier(identifier) => self.get_matcher_for_identifier(
                &identifier,
                struct_name,
                current_node,
                current_node_id,
                existing,
                query_values,
            ),
            unhandled => {
                log::warn!(
                    "Unhandled definition in language {}: {:#?}, {:#?}",
                    self.language.name(),
                    unhandled.kind_name(),
                    unhandled.source()
                );
                self.get_default_matcher(existing, query_values)
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
        let starting_node_id = format_ident!("id");
        let mut query_values = HashMap::new();
        let remaining_nodes = self.group_children(
            &self.node(),
            &self.node().children(self.tree).into_iter().next().unwrap(),
            &mut query_values,
            &starting_node,
            &starting_node_id,
        );
        return self._get_matcher_for_named_node(
            struct_name,
            &struct_name,
            kind,
            &starting_node,
            &starting_node_id,
            remaining_nodes,
            &mut query_values,
        );
    }
    fn target_captures(&self) -> Vec<&ts_query::Capture> {
        let mut captures: Vec<&ts_query::Capture> = self
            .captures()
            .into_iter()
            .filter(|c| !full_name_for_capture(c).contains("."))
            .collect();
        captures.sort_by_key(|c| full_name_for_capture(c));
        captures.dedup_by_key(|c| full_name_for_capture(c));
        captures
    }
    fn target_capture_names(&self) -> Vec<String> {
        self.target_captures()
            .into_iter()
            .map(|c| name_for_capture(c))
            .collect()
    }
    fn get_type_for_field(
        &self,
        parent: &ts_query::NamedNode,
        field: &ts_query::FieldDefinition,
    ) -> String {
        let parent_name = normalize_type_name(&parent.name(self.tree).source(), true);
        let field_name = normalize_field_name(
            &field
                .name(self.tree)
                .into_iter()
                .filter(|n| n.is_named())
                .next()
                .unwrap()
                .source(),
        );
        let parsed_field = self.get_field_for_field_name(&field_name, &parent_name);
        if let Some(parsed_field) = parsed_field {
            parsed_field.type_name()
        } else {
            panic!(
                "No field found for: {:#?}, {:#?}, {:#?}",
                field, field_name, parent_name
            );
        }
    }
    pub fn get_fields(&self) -> Vec<field::Field> {
        let mut fields = Vec::new();
        for capture in self.target_captures() {
            let name = name_for_capture(capture);
            let mut type_name = format_ident!("NodeTypes");
            for (node, id) in self.tree.descendants(&self.root_id) {
                if let ts_query::NodeTypesRef::Capture(other) = node.as_ref() {
                    if other == capture {
                        let mut preceding_siblings =
                            id.preceding_siblings(self.tree.arena()).skip(1);
                        while let Some(prev) = preceding_siblings.next() {
                            if let Some(prev_capture) = self.tree.arena().get(prev) {
                                match prev_capture.get().as_ref() {
                                    ts_query::NodeTypesRef::NamedNode(prev_capture) => {
                                        type_name = format_ident!(
                                            "{}",
                                            normalize_type_name(&prev_capture.source(), true)
                                        );
                                        break;
                                    }
                                    ts_query::NodeTypesRef::Identifier(prev_capture) => {
                                        type_name = format_ident!(
                                            "{}",
                                            normalize_type_name(&prev_capture.source(), true)
                                        );
                                        break;
                                    }
                                    ts_query::NodeTypesRef::AnonymousUnderscore(_) => {
                                        let mut ancestors = id.ancestors(self.tree.arena()).skip(2);
                                        if let Some(field) = ancestors.next() {
                                            if let Some(parent) = ancestors.next() {
                                                // Field definitions (example)
                                                // (new_expression\n  constructor: (_) @name) @reference.class
                                                let parent: &ts_query::NamedNode = self
                                                    .tree
                                                    .get(&parent)
                                                    .unwrap()
                                                    .as_ref()
                                                    .try_into()
                                                    .unwrap();

                                                let field: &ts_query::FieldDefinition = self
                                                    .tree
                                                    .get(&field)
                                                    .unwrap()
                                                    .as_ref()
                                                    .try_into()
                                                    .unwrap();
                                                type_name = format_ident!(
                                                    "{}",
                                                    self.get_type_for_field(parent, field)
                                                );
                                            }
                                        }
                                        break; // Could be any type
                                    }
                                    _ => {
                                        panic!("Unexpected capture: {:#?}", prev_capture);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            fields.push(field::Field {
                name: name.to_string(),
                kind: type_name.to_string(),
                is_optional: false,
                is_multiple: false,
                query: self.node().source().to_string(),
                is_subenum: self.state.get_subenum_struct_names().contains(&type_name),
            });
        }
        fields
    }
    fn category(&self) -> String {
        let name = self.name();
        let category = name
            .split(".")
            .next()
            .ok_or_else(|| {
                let msg = format!("No category found for: {}", name);
                Error::msg(msg)
            })
            .unwrap();
        pluralize(category.to_string().as_str(), 2, false)
    }

    pub fn symbol(&self) -> symbol::Symbol {
        symbol::Symbol {
            name: self.symbol_name().to_string(),
            type_name: self.struct_name().to_string(),
            language_struct: self.language.file_struct_name().to_string(),
            fields: self.get_fields(),
            category: self.category(),
            subcategory: self.executor_id().to_string(),
        }
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
    fn symbols<'a, 'db: 'a>(
        &'a self,
        db: &'db dyn salsa::Database,
    ) -> BTreeMap<String, symbol::Symbol> {
        let mut symbols = BTreeMap::new();
        for (name, query) in self.queries(db).into_iter() {
            if vec!["definitions".to_string(), "references".to_string()].contains(&query.category())
            {
                symbols.insert(name, query.symbol());
            }
        }
        symbols
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
