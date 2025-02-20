use codegen_sdk_common::{naming::normalize_type_name, parser::FieldDefinition};
use log::info;

#[derive(Debug)]
pub struct Field<'a> {
    raw: &'a FieldDefinition,
    name: String,
    node_name: String,
}

impl<'a> Field<'a> {
    pub fn new(node_name: &str, name: &str, raw: &'a FieldDefinition) -> Self {
        Self {
            node_name: node_name.to_string(),
            name: name.to_string(),
            raw,
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn normalized_name(&self) -> String {
        normalize_type_name(&self.name)
    }
    pub fn types(&self) -> Vec<String> {
        self.raw.types.iter().map(|t| t.type_name.clone()).collect()
    }
    pub fn type_name(&self) -> String {
        let types = self.types();
        if types.len() == 1 {
            types[0].clone()
        } else {
            format!("{}{}", self.node_name, self.normalized_name())
        }
    }
}
