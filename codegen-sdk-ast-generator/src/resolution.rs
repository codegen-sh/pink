use codegen_sdk_common::Language;

mod dependencies;
mod references;
mod scope;
mod stack;
pub fn generate_resolution(language: &Language) -> Vec<syn::Stmt> {
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
