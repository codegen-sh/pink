use std::collections::BTreeMap;

use codegen_sdk_common::{CSTNode, HasChildren, Language};
use codegen_sdk_cst::ts_query;
use convert_case::{Case, Casing};
use log::info;
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, format_ident, quote};

use super::query::Query;
pub fn generate_visitor(queries: &Vec<&Query>, language: &Language) -> TokenStream {
    let language_name = format_ident!("{}", language.name());
    let mut names = Vec::new();
    let mut types = Vec::new();
    let mut variants = Vec::new();
    let mut enter_methods = BTreeMap::new();
    for query in queries {
        names.push(query.executor_id());
        types.push(format_ident!("{}", query.struct_name()));
        for variant in query.struct_variants() {
            variants.push(format_ident!("{}", variant));
            enter_methods
                .entry(variant)
                .or_insert(Vec::new())
                .push(query);
        }
    }
    log::info!("Enter methods: {:#?}", enter_methods);
    let mut methods = Vec::new();
    for (variant, queries) in enter_methods {
        let mut matchers = TokenStream::new();
        let enter = format_ident!("enter_{}", variant.to_case(Case::Snake));
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
            fn #enter(&mut self, node: &codegen_sdk_cst::#language_name::#struct_name) {
                #matchers
            }
        });
    }
    let name = format_ident!("QueryExecutor");
    quote! {
        #[derive(Visitor, Default, Debug, Clone)]
        #[visitor(
            #(#language_name::#variants(enter)),*
        )]
        pub struct #name {
            #(pub #names: Vec<#language_name::#types>),*
        }
        impl #name {
            #(#methods)*
        }
    }
}

#[cfg(test)]
mod tests {
    use codegen_sdk_common::{Language, language::typescript::Typescript};
    use codegen_sdk_cst::query::{HasQuery, Query};

    use super::*;

    #[test_log::test]
    fn test_generate_visitor() {
        let language = &Typescript;
        let queries = language.definitions();
        let visitor = generate_visitor(&queries.values().collect());
        insta::assert_snapshot!(
            codegen_sdk_common::generator::format_code(&visitor.to_string()).unwrap()
        );
    }
}
