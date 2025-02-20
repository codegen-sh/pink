use codegen_sdk_common::naming::normalize_type_name;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use codegen_sdk_common::parser::TypeDefinition;
#[derive(Debug)]
pub struct Node<'a> {
    raw: &'a codegen_sdk_common::parser::Node,
    pub subenums: Vec<String>,
}
impl<'a> From<&'a codegen_sdk_common::parser::Node> for Node<'a> {
    fn from(raw: &'a codegen_sdk_common::parser::Node) -> Self {
        Node {
            raw,
            subenums: Vec::new(),
        }
    }
}
impl<'a> Node<'a> {
    pub fn normalize_name(&self) -> String {
        normalize_type_name(&self.raw.type_name)
    }
    pub fn add_subenum(&mut self, subenum: String) {
        self.subenums.push(subenum);
    }
    pub fn get_enum_tokens(&self) -> TokenStream {
        let name = format_ident!("{}", self.normalize_name());
        let subenum_names = &self.subenums.iter().map(|s| format_ident!("{}", normalize_type_name(s))).collect::<Vec<_>>();
        if subenum_names.is_empty() {
            quote! {
                #name(#name)
            }
        } else {
            quote! {
                #[subenum(#(#subenum_names), *)]
                #name(#name)
            }
        }
    }
    pub fn get_children_names(&self) -> Vec<String> {
        let mut children_names = Vec::new();
        if let Some(children) = &self.raw.children {
            children_names.extend(children.types.iter().map(|t| t.type_name.clone()));

        }
        if let Some(fields) = &self.raw.fields {
            for field in fields.fields.values() {
                children_names.extend(field.types.iter().map(|t| t.type_name.clone()));
            }
        }
        children_names
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_enum_tokens() {
        let base_node = codegen_sdk_common::parser::Node {
            type_name: "test".to_string(),
            subtypes: vec![],
            named: false,
            root: false,
            fields: None,
            children: None,
        };
        let mut node = Node::from(&base_node);

        let tokens = node.get_enum_tokens();
        assert_eq!(tokens.to_string(), quote! { Test(Test) }.to_string());
        node.add_subenum("subenum".to_string());
        let tokens = node.get_enum_tokens();
        assert_eq!(
            tokens.to_string(),
            quote! {
                #[subenum(Subenum)]
                Test (Test)
            }
            .to_string()
        );
    }
}
