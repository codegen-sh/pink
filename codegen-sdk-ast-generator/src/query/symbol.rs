use convert_case::{Case, Casing};
use pluralizer::pluralize;
use proc_macro2::Span;
use quote::format_ident;
use syn::parse_quote_spanned;

use crate::query::field::Field;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub type_name: String,
    pub category: String,
    pub subcategory: String,
    pub language_struct: String,
    pub fields: Vec<Field>,
}
impl Symbol {
    pub fn as_syn_struct(&self) -> Vec<syn::Stmt> {
        let span = Span::call_site();
        let variant = syn::Ident::new(&self.name, span);
        let type_name = syn::Ident::new(&self.type_name, span);
        let language_struct = syn::Ident::new(&self.language_struct, span);
        let fields = self
            .fields
            .iter()
            .map(|field| field.as_syn_field())
            .collect::<Vec<_>>();
        let getters = self
            .fields
            .iter()
            .map(|field| field.getter())
            .collect::<Vec<_>>();
        parse_quote_spanned! {
            span =>
            #[salsa::tracked]
            pub struct #variant<'db> {
                #[id]
                _fully_qualified_name: codegen_sdk_resolution::FullyQualifiedName,
                #[id]
                pub node_id: codegen_sdk_common::CSTNodeTreeId,
                // #[tracked]
                // #[return_ref]
                // pub node: crate::cst::#type_name<'db>,
                #(#fields),*
            }
            impl<'db> #variant<'db> {
                pub fn node(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &'db crate::cst::#type_name<'db> {
                    let file = self.file(db);
                    let tree = file.tree(db);
                    tree.get(self.node_id(db).id(db)).unwrap().as_ref().try_into().unwrap()
                }
                #(#getters)*
            }
            impl<'db> codegen_sdk_resolution::HasFile<'db> for #variant<'db> {
                type File<'db1> = #language_struct<'db1>;
                fn file(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &'db Self::File<'db> {
                    let file = self._fully_qualified_name(db).file(db);
                    parse(db, file)
                }
                fn root_path(&self, db: &'db dyn codegen_sdk_resolution::Db) -> &PathBuf {
                    self.node_id(db).root(db).path(db)
                }
            }
            impl<'db> codegen_sdk_resolution::HasId<'db> for #variant<'db> {
                fn fully_qualified_name(&self, db: &'db dyn salsa::Database) -> codegen_sdk_resolution::FullyQualifiedName {
                    self._fully_qualified_name(db)
                }
            }
        }
    }
    pub fn py_file_getter(&self) -> syn::Stmt {
        let span = Span::call_site();
        let method_name = syn::Ident::new(
            &pluralize(self.name.to_case(Case::Snake).as_str(), 2, false),
            span,
        );
        let category = syn::Ident::new(&self.category, span);
        let type_name = syn::Ident::new(&self.name, span);
        let subcategory = syn::Ident::new(&self.subcategory, span);
        parse_quote_spanned! {
            span =>
            #[getter]
            pub fn #method_name(&self, py: Python<'_>) -> PyResult<Vec<#type_name>> {
                let file = self.file(py)?;
                let db = self.codebase.get(py).db();
                let category = file.#category(db);
                let subcategory = category.#subcategory(db);
                let nodes = subcategory.values().map(|values| values.into_iter().enumerate().map(|(idx, node)| #type_name::new(node.fully_qualified_name(db), idx, self.codebase.clone()))).flatten().collect();
                Ok(nodes)
            }

        }
    }
    pub fn py_file_get(&self) -> syn::Stmt {
        let span = Span::call_site();
        let method_name = format_ident!("get_{}", self.name.to_case(Case::Snake));
        let category = syn::Ident::new(&self.category, span);
        let type_name = syn::Ident::new(&self.name, span);
        let subcategory = syn::Ident::new(&self.subcategory, span);
        parse_quote_spanned! {
            span =>
            #[pyo3(signature = (name,optional=false))]
            pub fn #method_name(&self, py: Python<'_>, name: String, optional: bool) -> PyResult<Option<#type_name>> {
                let file = self.file(py)?;
                let db = self.codebase.get(py).db();
                let category = file.#category(db);
                let subcategory = category.#subcategory(db);
                let res = subcategory.get(&name);
                if let Some(nodes) = res {
                    if nodes.len() == 1 {
                        Ok(Some(#type_name::new(nodes[0].fully_qualified_name(db), 0, self.codebase.clone())))
                    } else {
                        Err(pyo3::exceptions::PyValueError::new_err(format!("Ambiguous symbol {} found {} possible matches", name, nodes.len())))
                    }
                } else {
                    if optional {
                        Ok(None)
                    } else {
                        Err(pyo3::exceptions::PyValueError::new_err(format!("No symbol {} found", name)))
                    }
                }
            }

        }
    }
}
