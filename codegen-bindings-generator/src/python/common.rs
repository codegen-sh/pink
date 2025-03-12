use codegen_sdk_common::{Language, generator::format_code};
use quote::{ToTokens, format_ident};
use syn::parse_quote;
pub fn generate_python_bindings_common(languages: &Vec<&Language>) -> anyhow::Result<()> {
    let variants: Vec<syn::Variant> = languages
        .iter()
        .filter(|language| language.name() != "ts_query")
        .map(|language| {
            let flag_name = language.name();
            let struct_name = format_ident!("{}", language.struct_name());
            let name = format_ident!("{}", language.name());
            let file_type = format_ident!("{}", language.file_struct_name());
            parse_quote! {
                #[cfg(feature = #flag_name)]
                #struct_name(#name::#file_type)
            }
        })
        .collect();
    let modules: Vec<syn::ItemMod> = languages
        .iter()
        .filter(|language| language.name() != "ts_query")
        .map(|language| {
            let flag_name = language.name();
            let name = format_ident!("{}", language.name());
            let path = format!("/{}-bindings.rs", language.name());
            parse_quote! {
                #[cfg(feature = #flag_name)]
                #[allow(unused)]
                pub mod #name {
                    include!(concat!(env!("OUT_DIR"), #path));
                }
            }
        })
        .collect();
    let parsers: Vec<syn::Stmt> = languages
        .iter()
        .filter(|language| language.name() != "ts_query")
        .map(|language| -> Vec<syn::Stmt> {
            let flag_name = language.name();
            let name = format_ident!("{}", language.name());
            let variant = format_ident!("{}", language.struct_name());
            let file_name = format_ident!("{}", language.file_struct_name());
            parse_quote! {
                #[cfg(feature = #flag_name)]
                if codegen_sdk_common::language::#name::#variant.should_parse(path).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))? {
                    let file = #name::#file_name::new(path.clone(), codebase);
                    return Ok(FileEnum::#variant(file));
                }
            }
        })
        .flatten()
        .collect();
    let parse: syn::Stmt = parse_quote! {
        pub fn parse(path: &PathBuf, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> PyResult<FileEnum> {
            #(#parsers)*
        let file = crate::file::File::new(path.clone(),   codebase);
        Ok(FileEnum::Unknown(file))
    }
    };
    let ast: syn::File = parse_quote! {
        #(#modules)*
        #[derive(IntoPyObject)]
        enum FileEnum {
            #(#variants,)*
            Unknown(File),
        }
        impl FileEnum {
            #parse
        }
    };
    let out_dir = std::env::var("OUT_DIR")?;
    let out_file = format!("{}/common-bindings.rs", out_dir);
    std::fs::write(&out_file, ast.to_token_stream().to_string())?;
    let ast = format_code(&ast)
        .unwrap_or_else(|_| panic!("Failed to format common bindings at {}", out_file));
    std::fs::write(out_file, ast)?;
    Ok(())
}
