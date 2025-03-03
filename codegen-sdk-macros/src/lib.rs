#![feature(extend_one)]
extern crate proc_macro;
use codegen_sdk_common::language::LANGUAGES;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

// #[proc_macro]
// pub fn parse_language(_item: TokenStream) -> TokenStream {
//     let target_language = _item.to_string();
//     let language = get_language(&target_language);
//     format!(
//         "#[cfg(feature = \"{name}\")]
//     if {name}::{struct_name}::should_parse(file_path)? {{
//         let parsed = {name}::{struct_name}::parse_file(db, file_path, #[cfg(feature = \"serialization\")] cache)?;
//         #[cfg(feature = \"serialization\")] {{
//             log::debug!(\"Serializing {name}\");
//             let writer = cache.get_writer(&serialized_path)?;
//             let _ = rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(&parsed, writer)?;
//         }}
//         return Ok(Box::new(parsed));
//     }}
//  ",
//         name = language.name(),
//         struct_name = language.struct_name
//     )
//     .parse()
//     .unwrap()
// }
// #[proc_macro]
// pub fn parse_languages(_item: TokenStream) -> TokenStream {
//     let mut output = String::new();
//     output.push_str("use codegen_sdk_macros::parse_language;");
//     for language in LANGUAGES.iter() {
//         output.push_str(&format!("parse_language!({});", language.name()));
//     }
//     output.parse().unwrap()
// }

#[proc_macro]
pub fn languages_ast(_item: TokenStream) -> TokenStream {
    let mut output = Vec::new();
    for language in LANGUAGES.iter() {
        if language.name() == "ts_query" {
            continue;
        }
        let name = language.name();
        let package_name = format_ident!("codegen_sdk_{}", name);
        let struct_name = format_ident!("{}", language.struct_name);
        let file_name = format_ident!("{}File", language.struct_name);
        let variant: proc_macro2::TokenStream = quote! {
        #[cfg(feature = #name)]
        #struct_name(#package_name::ast::#file_name<'db>),
        };
        output.push(variant);
    }
    let enum_output: TokenStream = quote! {
    #[derive(Debug, Clone, Eq, PartialEq, Hash, salsa::Update)]
    pub enum ParsedFile<'db> {
        #(#output)*
    }
    }
    .into();
    enum_output
}

#[proc_macro]
pub fn parse_language(_item: TokenStream) -> TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    for language in LANGUAGES.iter() {
        if language.name() == "ts_query" {
            continue;
        }
        let name = language.name();
        let package_name = format_ident!("codegen_sdk_{}", name);
        let struct_name = format_ident!("{}", language.struct_name);
        let variant: proc_macro2::TokenStream = quote! {
            #[cfg(feature = #name)]
            if #package_name::cst::#struct_name::should_parse(&file.path(db)).unwrap_or(false) {
                let parsed = #package_name::ast::parse(db, file);
                return Parsed::new(
                    db,
                    Some(ParsedFile::#struct_name(parsed)),
                );
            }
        };
        output.extend_one(variant);
    }
    output.into()
}
