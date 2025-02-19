use std::collections::HashMap;

use codegen_sdk_common::parser::TypeDefinition;
use proc_macro2::TokenStream;
#[derive(Default, Debug)]
pub struct State {
    pub enums: TokenStream,
    pub structs: String,
    pub variants: HashMap<String, Vec<TypeDefinition>>,
    pub anonymous_nodes: HashMap<String, String>,
}
