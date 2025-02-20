use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn get_from_for_enum(variant: &str, enum_name: &str) -> TokenStream {
    let enum_name = format_ident!("{}", enum_name);
    let variant = format_ident!("{}", variant);
    quote! {
        impl std::convert::From<#variant> for #enum_name {
            fn from(variant: #variant) -> Self {
                Self::#variant(variant)
            }
        }
    }
}
pub fn get_serialize_bounds() -> TokenStream {
    quote! {
       #[derive(Debug, Clone, Deserialize, Archive, Serialize)]
       #[rkyv(serialize_bounds(
           __S: rkyv::ser::Writer + rkyv::ser::Allocator,
           __S::Error: rkyv::rancor::Source,
       ))]
       #[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]
       #[rkyv(bytecheck(
           bounds(
               __C: rkyv::validation::ArchiveContext,
               __C::Error: rkyv::rancor::Source,
           )
       ))]
    }
}
