use codegen_sdk_common::Language;
use quote::format_ident;
use syn::parse_quote;
pub(crate) fn get_dependencies_name(language: &Language) -> syn::Ident {
    format_ident!("{}Dependencies", language.struct_name())
}

pub fn generate_dependencies(language: &Language) -> Vec<syn::Stmt> {
    let name = get_dependencies_name(language);
    let language_name = format_ident!("{}", language.struct_name());
    let language_submodule = format_ident!("{}", language.name());
    parse_quote!(
        #[salsa::tracked]
        pub struct #name<'db> {
            #[id]
            id: codegen_sdk_common::FileNodeId,
            #[return_ref]
            #[tracked]
            #[no_eq]
            pub dependencies: codegen_sdk_common::hash::FxHashMap<
                codegen_sdk_resolution::FullyQualifiedName,
                codegen_sdk_common::hash::FxIndexSet<crate::ast::Reference<'db>>,
            >,
        }
        impl<'db>
        codegen_sdk_resolution::Dependencies<
            'db,
            codegen_sdk_resolution::FullyQualifiedName,
            crate::ast::Reference<'db>,
        > for #name<'db>
    {
        fn get(
            &'db self,
            db: &'db dyn codegen_sdk_resolution::Db,
            key: &codegen_sdk_resolution::FullyQualifiedName,
        ) -> Option<&'db codegen_sdk_common::hash::FxIndexSet<crate::ast::Reference<'db>>> {
            self.dependencies(db).get(key)
        }
    }
    #[salsa::tracked(return_ref, no_eq)]
    pub fn dependencies<'db>(
        db: &'db dyn codegen_sdk_resolution::Db,
        input: codegen_sdk_common::FileNodeId,
    ) -> #name<'db> {
        let file = parse(db, input);
        #name::new(db, file.id(db), file.compute_dependencies(db))
    }

    #[salsa::tracked(return_ref, no_eq)]
    pub fn dependency_matrix<'db>(
        db: &'db dyn codegen_sdk_resolution::Db,
    ) -> codegen_sdk_common::hash::FxIndexMap<
        codegen_sdk_resolution::FullyQualifiedName,
        codegen_sdk_common::hash::FxIndexSet<codegen_sdk_common::FileNodeId>,
    > {
        let mut ret: codegen_sdk_common::hash::FxIndexMap<
            codegen_sdk_resolution::FullyQualifiedName,
            codegen_sdk_common::hash::FxIndexSet<codegen_sdk_common::FileNodeId>,
        > = Default::default();
        let files = codegen_sdk_resolution::files(db)
            .into_iter()
            .filter(|file| {
                codegen_sdk_common::language::#language_submodule::#language_name
                    .should_parse(file.path(db))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let dependencies: Vec<
            Vec<(
                codegen_sdk_resolution::FullyQualifiedName,
                codegen_sdk_common::FileNodeId,
            )>,
        > = salsa::par_map(db, files, |db, file| {
            let dependencies = dependencies(db, file.clone());
            let mut ret = Vec::default();
            for name in dependencies.dependencies(db).keys() {
                ret.push((name.clone(), file.clone()));
            }
            ret
        });
        for (name, dependencies) in dependencies.into_iter().flatten() {
            ret.entry(name).or_default().insert(dependencies);
        }
        ret
    }
    impl<'db> codegen_sdk_resolution::Compute<'db> for crate::cst::#language_name {
        fn compute(db: &'db dyn codegen_sdk_resolution::Db) {
            dependency_matrix(db);
        }
    }
    )
}
