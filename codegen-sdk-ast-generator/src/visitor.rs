use std::collections::{BTreeMap, BTreeSet};

use codegen_sdk_common::{CSTNode, HasChildren, Language};
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
            let node = query.node();
            for child in node.children() {
                info!("child kind:{} source:{}", child.kind(), child.source());
                if let ts_query::NamedNodeChildren::FieldDefinition(field_definition) = child {
                    let field_name = &field_definition.name;
                    let source = field_definition.source();
                    let children = &field_definition.children();
                    info!("source: {:?}", source);
                    info!("field_name: {:?}", field_name);
                    info!("children: {:?}", children);
                }
            }
        }
        methods.push(quote! {
            fn #enter(&mut self, node: &crate::cst::#struct_name<'db>) {
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
            #(#[visit(drive(&crate::cst::#nodes<'db>))])*
            #(#[visit(drive(&crate::cst::#variants<'db>))])*
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
    quote! {
        #[derive(Visitor, Visit, Debug, Clone, Eq, PartialEq, salsa::Update, Hash, Default)]
        #visitor
        pub struct #name<'db> {
            #(pub #names: BTreeMap<String, Vec<crate::cst::#types<'db>>>,)*
            phantom: std::marker::PhantomData<&'db ()>,
        }
        impl<'db> #name<'db> {
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
