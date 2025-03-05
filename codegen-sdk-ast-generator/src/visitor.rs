use std::collections::{BTreeMap, BTreeSet};

use codegen_sdk_common::{CSTNode, HasChildren, Language, naming::normalize_type_name};
use codegen_sdk_ts_query::cst as ts_query;
use convert_case::{Case, Casing};
use log::info;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::query::Query;
use crate::query::HasQuery;
pub fn generate_visitor<'db>(
    db: &'db dyn salsa::Database,
    language: &Language,
    name: &str,
) -> TokenStream {
    log::info!(
        "Generating visitor for language: {} for {}",
        language.name(),
        name
    );
    let raw_queries = language.queries_with_prefix(db, &format!("{}", name));
    let queries: Vec<&Query> = raw_queries.values().flatten().collect();
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut variants = BTreeSet::new();
    let mut enter_methods = BTreeMap::new();
    for query in queries {
        names.push(query.executor_id());
        types.push(format_ident!("{}", query.struct_name()));
        for variant in query.struct_variants() {
            variants.insert(format_ident!("{}", variant));
            enter_methods
                .entry(variant)
                .or_insert(Vec::new())
                .push(query);
        }
    }
    let mut methods = Vec::new();
    for (variant, queries) in enter_methods {
        let mut matchers = TokenStream::new();
        let enter = format_ident!("enter_{}", variant);
        let struct_name = format_ident!("{}", variant);
        for query in queries {
            matchers.extend_one(query.matcher(&variant));
        }
        methods.push(quote! {
            fn #enter<'db2>(&self, node: &'db2 crate::cst::#struct_name<'db>) where 'db2: 'db {
                #matchers
            }
        });
    }
    let visitor = if variants.len() > 0 {
        let first_query = raw_queries.values().flatten().next().unwrap();
        let state = first_query.state.clone();
        let mut nodes = BTreeSet::new();
        nodes.extend(state.get_node_struct_names());
        nodes.extend(state.get_subenum_struct_names());
        nodes = nodes.difference(&variants).cloned().collect();
        quote! {
            #(#[visit(drive(&'db1 crate::cst::#nodes<'db>))])*
            #(#[visit(drive(&'db1 crate::cst::#variants<'db>))])*
            #(#[visit(drive(crate::cst::#nodes<'db>))])*
            #[visit(drive(for<T> Box<T>))]
            #[visit(drive(for<T> Vec<T>))]
            #[visit(drive(for<T> Option<T>))]
            #[visit(
                #(enter(#variants:crate::cst::#variants<'db>)),*
            )]
        }
    } else {
        quote! {}
    };
    let name = format_ident!("{}s", name.to_case(Case::Pascal));
    let visitor_name = format_ident!("{}Visitor", name);
    let root_name = format_ident!("{}", normalize_type_name(&language.root_node(), true));
    let i = (0..types.len()).map(syn::Index::from);
    let sender_param = quote! {
        #(Sender<(String, &'db1 crate::cst::#types<'db>)>),*
    };
    let visitor_constructor = quote! {
        fn new(sender: (#sender_param)) -> Self {
            Self {
                #(#names: sender.#i,)*
                phantom: std::marker::PhantomData,
                phantom2: std::marker::PhantomData,
            }
        }
    };
    let senders = types
        .iter()
        .map(|t| format_ident!("sender_{}", t))
        .collect::<Vec<_>>();
    let receivers = types
        .iter()
        .map(|t| format_ident!("receiver_{}", t))
        .collect::<Vec<_>>();
    let empty_constructors = types
        .iter()
        .map(|_| quote! {Default::default()})
        .collect::<Vec<_>>();
    let output_constructor = quote! {
        pub fn visit(db: &'db dyn salsa::Database, root: &'db crate::cst::#root_name<'db>) -> Self {
            #(let (#senders, #receivers) = std::sync::mpsc::channel();)*
            let visitor = #visitor_name::new(
                (
                    #(#senders),*
                )
            );
            visitor.visit_by_val_infallible(root);

            #(
                let mut #names: BTreeMap<String, Vec<&'db crate::cst::#types<'db>>> = BTreeMap::new();
                while let Ok(val) = #receivers.recv() {
                    let (name, node) = val;
                    #names.entry(name).or_default().push(node);
                }
            )*
            Self {
                #(#names),*
            }
        }
    };
    quote! {
        // Three lifetimes:
        // db: the lifetime of the database
        // db1: the lifetime of the visitor executing per-node
        // db2: the lifetime of the references held by the visitor
        #[derive(Debug, Clone, Eq, PartialEq, Hash, Default, salsa::Update)]
        pub struct #name<'db> {

            #(
                pub #names: BTreeMap<String, Vec<&'db crate::cst::#types<'db>>>,
            )*
        }
        impl<'db> #name<'db> {
            #output_constructor
        }

        #[derive(Visitor, Visit, Debug)]
        #visitor
        pub struct #visitor_name<'db, 'db1> where 'db1: 'db {
            #(pub #names: Sender<(String, &'db1 crate::cst::#types<'db>)>,)*
            phantom: std::marker::PhantomData<&'db ()>,
            phantom2: std::marker::PhantomData<&'db1 ()>,
        }
        impl<'db, 'db1> #visitor_name<'db, 'db1> {
            #visitor_constructor
            #(#methods)*
        }
    }
}

#[cfg(all(test))]
mod tests {
    use codegen_sdk_common::language::{python::Python, typescript::Typescript};
    use rstest::rstest;

    use super::*;

    #[test_log::test(rstest)]
    #[case::typescript(&Typescript)]
    #[case::python(&Python)]
    fn test_generate_visitor(#[case] language: &Language) {
        let db = codegen_sdk_cst::CSTDatabase::default();
        let visitor = generate_visitor(&db, language, "definition");
        insta::assert_snapshot!(
            format!("{}", language.name()),
            codegen_sdk_common::generator::format_code_string(&visitor.to_string()).unwrap()
        );
    }
}
