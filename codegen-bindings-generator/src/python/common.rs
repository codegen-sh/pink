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
                pub mod #name {
                    include!(concat!(env!("OUT_DIR"), #path));
                }
            }
        })
        .collect();

    let ast: syn::File = parse_quote! {
        #(#modules)*
        #[derive(IntoPyObject)]
        enum FileEnum {
            #(#variants,)*
            Unknown(File),
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
