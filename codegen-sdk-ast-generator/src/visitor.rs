use std::collections::{BTreeMap, BTreeSet};

use codegen_sdk_common::Language;
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse_quote_spanned;

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
    let mut symbol_names = Vec::new();
    for query in queries {
        names.push(query.executor_id());
        types.push(format_ident!("{}", query.struct_name()));
        symbol_names.push(query.symbol_name());
        for variant in query.struct_variants() {
            variants.insert(format_ident!("{}", variant));
            enter_methods
                .entry(variant)
                .or_insert(Vec::new())
                .push(query);
        }
    }
    let mut methods: Vec<syn::Arm> = Vec::new();
    for (variant, queries) in enter_methods.iter() {
        let mut matchers = TokenStream::new();
        let struct_name = format_ident!("{}", variant);
        for query in queries {
            matchers.extend_one(query.matcher(&variant));
        }
        let span = Span::mixed_site();
        methods.push(parse_quote_spanned! { span =>
            crate::cst::NodeTypes::#struct_name(node) => {
                #matchers
            }
        });
    }

    let symbol_name = if name == "definition" {
        format_ident!("Symbol")
    } else {
        format_ident!("Reference")
    };
    let maps = quote! {
        #(
            let mut #names: BTreeMap<String,Vec<#symbol_names<'db>>> = BTreeMap::new();
        )*
    };
    let constructor = quote! {
        Self::new(db, #(#names),*)

    };
    let mut defs = Vec::new();
    let language_struct = format_ident!("{}File", language.struct_name());
    for (_, type_name) in symbol_names.iter().zip(types.iter()) {
        let query = enter_methods
            .get(&type_name.to_string())
            .unwrap()
            .first()
            .unwrap();
        let symbol = query.symbol();
        defs.extend(symbol.as_syn_struct());
    }
    let symbol = if defs.len() > 0 {
        quote! {
                #(
                    #defs
                )*
            #[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
            pub enum #symbol_name<'db> {
                #(
                    #symbol_names(#symbol_names<'db>),
                )*
            }
            impl<'db> codegen_sdk_resolution::HasFile<'db> for #symbol_name<'db> {
                type File<'db1> = #language_struct<'db1>;
                fn file(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &'db Self::File<'db> {
                    match self {
                        #(Self::#symbol_names(symbol) => symbol.file(db),)*
                    }
                }
                fn root_path(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &PathBuf {
                    match self {
                        #(Self::#symbol_names(symbol) => symbol.root_path(db),)*
                    }
                }
            }
            impl<'db> codegen_sdk_resolution::HasId<'db> for #symbol_name<'db> {
                fn fully_qualified_name(&self, db: &'db dyn salsa::Database) -> codegen_sdk_resolution::FullyQualifiedName {
                    match self {
                        #(Self::#symbol_names(symbol) => symbol.fully_qualified_name(db),)*
                    }
                }
            }
        }
    } else {
        quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
            pub enum #symbol_name<'db> {
                _Phantom(std::marker::PhantomData<&'db ()>)
            }
        }
    };
    let name = format_ident!("{}s", name.to_case(Case::Pascal));
    let output_constructor = quote! {
        pub fn visit(db: &'db dyn salsa::Database, root: &'db crate::cst::Parsed<'db>) -> Self {
            #maps
            let tree = root.tree(db);
            for (node, id) in tree.descendants(&root.program(db)) {
                match node {
                    #(#methods,)*
                    _ => {}
                }
            }
            #constructor
        }
        pub fn default(db: &'db dyn salsa::Database) -> Self {
            #maps
            #constructor
        }
    };

    quote! {
        #symbol
        // Three lifetimes:
        // db: the lifetime of the database
        // db1: the lifetime of the visitor executing per-node
        // db2: the lifetime of the references held by the visitor
        #[salsa::tracked]
        pub struct #name<'db> {
            #(
                #[return_ref]
                pub #names: BTreeMap<String, Vec<#symbol_names<'db>>>,
            )*
        }
        impl<'db> #name<'db> {
            #output_constructor
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
