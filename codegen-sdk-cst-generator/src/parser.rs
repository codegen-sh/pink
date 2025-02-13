use std::error::Error;

use codegen_sdk_common::language::Language;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "type")]
    pub type_name: String,
    pub named: bool,
    #[serde(default)]
    pub subtypes: Vec<TypeDefinition>,
    #[serde(default)]
    pub fields: Option<Fields>,
    #[serde(default)]
    pub children: Option<Children>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fields {
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, FieldDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub multiple: bool,
    pub required: bool,
    #[serde(default)]
    pub types: Vec<TypeDefinition>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TypeDefinition {
    #[serde(rename = "type")]
    pub type_name: String,
    pub named: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Children {
    pub multiple: bool,
    pub required: bool,
    #[serde(default)]
    pub types: Vec<TypeDefinition>,
}

pub fn parse_node_types(language: &Language) -> Result<Vec<Node>, Box<dyn Error>> {
    let parsed: Vec<Node> = serde_json::from_str(language.node_types)?;
    Ok(parsed)
}
#[cfg(test)]
mod tests {
    use super::*;
    use codegen_sdk_common::language::python::Python;
    #[test]
    fn test_parse_node_types() {
        let cst = parse_node_types(&Python).unwrap();
        assert!(!cst.is_empty());
    }
}
