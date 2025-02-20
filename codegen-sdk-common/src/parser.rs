use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "type")]
    pub type_name: String,
    pub named: bool,
    #[serde(default)]
    pub root: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
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

pub fn parse_node_types(node_types: &str) -> anyhow::Result<Vec<Node>> {
    let parsed: Vec<Node> = serde_json::from_str(node_types)?;
    Ok(parsed)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::python::Python;
    #[test_log::test]
    fn test_parse_node_types() {
        let cst = parse_node_types(Python.node_types).unwrap();
        assert!(!cst.is_empty());
    }
}
