use codegen_sdk_common::Language;
use syn::parse_quote;

use super::dependencies::get_dependencies_name;

pub fn generate_references(language: &Language) -> Vec<syn::Stmt> {
    let dependencies_name = get_dependencies_name(language);
    let file_name = language.file_struct_name();
    parse_quote! {
        pub fn references_impl<'db>(
            db: &'db dyn codegen_sdk_resolution::Db,
            name: codegen_sdk_resolution::FullyQualifiedName,
        ) -> Vec<crate::ast::Reference<'db>> {
            let mut results = Vec::new();
            let dependency_matrix = dependency_matrix(db);
            let files = dependency_matrix.get(&name);
            if let Some(files) = files {
                log::info!(target: "resolution", "Finding references across {:?} files", files.len());
                for input in files.into_iter() {
                    let dependencies = dependencies(db, input.clone());
                    if let Some(references) = dependencies.get(db, &name) {
                        results.extend(references.iter().cloned());
                    }
                }
            }
            results
        }
        #[salsa::tracked]
        impl<'db>
            codegen_sdk_resolution::References<
                'db,
                #dependencies_name<'db>,
                crate::ast::Reference<'db>,
                #file_name<'db>,
            > for crate::ast::Symbol<'db>
        {
            fn references(self, db: &'db dyn codegen_sdk_resolution::Db) -> Vec<crate::ast::Reference<'db>> {
                let mut results = Vec::new();
                for reference in references_impl(db, self.fully_qualified_name(db)).iter() {
                    let resolved_stacks = reference.resolve_type(db);
                    if resolved_stacks
                        .iter()
                        .any(|stack| stack.entries(db).iter().any(|entry| *entry == self))
                    {
                        results.push(reference.clone());
                    }
                }
                results
            }
        }
    }
}
