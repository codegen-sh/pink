use std::collections::HashMap;

use codegen_sdk_common::parser::TypeDefinition;

#[derive(Default, Debug)]
pub struct State {
    pub enums: String,
    pub structs: String,
    pub variants: HashMap<String, Vec<TypeDefinition>>,
    pub anonymous_nodes: HashMap<String, String>,
}
