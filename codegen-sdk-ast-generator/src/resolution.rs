use codegen_sdk_common::Language;

mod dependencies;
mod references;
mod scope;
mod stack;
use quote::format_ident;
use syn::parse_quote;
pub fn generate_resolution(language: &Language) -> Vec<syn::Stmt> {
    if !language.tag_query.contains("@definition") && !language.tag_query.contains("@reference") {
        let language_struct_name = format_ident!("{}", language.struct_name());
        return parse_quote! {
            impl<'db> codegen_sdk_resolution::Compute<'db> for crate::cst::#language_struct_name {
                fn compute(db: &'db dyn codegen_sdk_resolution::Db) {
                }
            }
        };
    }
    let mut stmts = Vec::new();
    let stack = stack::generate_stack(&language);
    stmts.extend(stack);
    let dependencies = dependencies::generate_dependencies(&language);
    stmts.extend(dependencies);
    let references = references::generate_references(&language);
    stmts.extend(references);
    let scope = scope::generate_scope(&language);
    stmts.extend(scope);
    stmts
}
