use codegen_sdk_common::{naming::normalize_type_name, parser::FieldDefinition};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
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
            normalize_type_name(&types[0])
        } else {
            format!("{}{}", self.node_name, self.normalized_name())
        }
    }
    pub fn get_constructor_field(&self) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.normalized_name());
        let converted_type_name = format_ident!("{}", self.type_name());
        let original_name = self.name();
        if self.raw.multiple {
            quote! {
                #field_name_ident: get_multiple_children_by_field_name(&node, #original_name, buffer)?
            }
        } else if !self.raw.required {
            quote! {
                #field_name_ident: get_optional_child_by_field_name(&node, #original_name, buffer)?
            }
        } else {
            quote! {
                #field_name_ident: get_child_by_field_name(&node, #original_name, buffer)?
            }
        }
    }
    pub fn get_convert_child(&self) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.normalized_name());
        if self.raw.multiple {
            quote! {
                &Self::Child::try_from(Types::from((*child).clone())).unwrap()
            }
        } else if !self.raw.required {
            quote! {
                &Self::Child::try_from(Types::from((*child).clone())).unwrap()
            }
        } else {
            quote! {
                &Self::Child::try_from(Types::from((*self.#field_name_ident).clone())).unwrap()
            }
        }
    }
    pub fn get_children_field(&self) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.normalized_name());
        let convert_child = self.get_convert_child();
        if self.raw.multiple {
            quote! {
                children.extend(self.#field_name_ident.iter().map(|child|#convert_child));
            }
        } else if !self.raw.required {
            quote! {
                if let Some(child) = *self.#field_name_ident {
                    children.push(#convert_child);
                }
            }
        } else {
            quote! {
                children.push(#convert_child);
            }
        }
    }
    pub fn get_children_by_field_name_field(&self) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.normalized_name());
        let field_name = format_ident!("{}", self.name);
        let convert_child = self.get_convert_child();
        if self.raw.multiple {
            quote! {
                #field_name => self.#field_name_ident.iter().map(|child|#convert_child).collect()
            }
        } else if !self.raw.required {
            quote! {
                #field_name => self.#field_name_ident.map_or_else(|| vec![], |child| vec![#convert_child])
            }
        } else {
            quote! {
                #field_name => vec![#convert_child]
            }
        }
    }
    pub fn get_struct_field(&self) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.normalized_name());
        let converted_type_name = format_ident!("{}", self.type_name());
        if self.raw.multiple {
            quote! {
                pub #field_name_ident: Vec<#converted_type_name>
            }
        } else if !self.raw.required {
            quote! {
                pub #field_name_ident: Option<#converted_type_name>
            }
        } else {
            quote! {
                pub #field_name_ident: #converted_type_name
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use codegen_sdk_common::parser::TypeDefinition;

    use super::*;

    fn create_test_field_definition(name: &str, multiple: bool, required: bool) -> FieldDefinition {
        FieldDefinition {
            types: vec![TypeDefinition {
                type_name: name.to_string(),
                named: true,
            }],
            multiple,
            required,
        }
    }
    fn create_test_field_definition_variants(
        name: &Vec<String>,
        multiple: bool,
        required: bool,
    ) -> FieldDefinition {
        FieldDefinition {
            types: name
                .iter()
                .map(|n| TypeDefinition {
                    type_name: n.to_string(),
                    named: true,
                })
                .collect(),
            multiple,
            required,
        }
    }

    #[test]
    fn test_field_normalized_name() {
        let field_definition = create_test_field_definition("test_type", false, true);
        let field = Field::new("node", "field", &field_definition);
        assert_eq!(field.normalized_name(), "Field");
    }

    #[test]
    fn test_field_types() {
        let field_definition = create_test_field_definition_variants(
            &vec!["type_a".to_string(), "type_b".to_string()],
            false,
            true,
        );
        let field = Field::new("test_node", "test_field", &field_definition);
        assert_eq!(field.types(), vec!["type_a", "type_b"]);
    }

    #[test]
    fn test_field_type_name() {
        let field_definition = create_test_field_definition_variants(
            &vec!["test_type".to_string(), "test_type".to_string()],
            false,
            true,
        );
        let field = Field::new("Node", "field", &field_definition);
        assert_eq!(field.type_name(), "NodeField");
    }

    #[test]
    fn test_get_struct_field() {
        let field_definition = create_test_field_definition("test_type", false, true);
        let field = Field::new("test_node", "test_field", &field_definition);

        assert_eq!(
            field.get_struct_field().to_string(),
            quote!(pub TestField: TestType).to_string()
        );

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new("test_node", "test_field", &optional_definition);

        assert_eq!(
            optional_field.get_struct_field().to_string(),
            quote!(pub TestField: Option<TestType>).to_string()
        );

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new("test_node", "test_field", &multiple_definition);

        assert_eq!(
            multiple_field.get_struct_field().to_string(),
            quote!(pub TestField: Vec<TestType>).to_string()
        );
    }

    #[test]
    fn test_get_constructor_field() {
        let field_definition = create_test_field_definition("test_type", false, true);
        let field = Field::new("test_node", "test_field", &field_definition);

        assert_eq!(
            field.get_constructor_field().to_string(),
            quote!(TestField: get_child_by_field_name(&node, "test_field", buffer)?).to_string()
        );

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new("test_node", "test_field", &optional_definition);

        assert_eq!(
            optional_field.get_constructor_field().to_string(),
            quote!(TestField: get_optional_child_by_field_name(&node, "test_field", buffer)?)
                .to_string()
        );

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new("test_node", "test_field", &multiple_definition);

        assert_eq!(
            multiple_field.get_constructor_field().to_string(),
            quote!(TestField: get_multiple_children_by_field_name(&node, "test_field", buffer)?)
                .to_string()
        );
    }

    #[test]
    fn test_get_children_field() {
        // Test required field
        let field_definition = create_test_field_definition("test_type", false, true);
        let field = Field::new("test_node", "test_field", &field_definition);

        assert_eq!(
            field.get_children_field().to_string(),
            quote!(children.push(&Self::Child::try_from(Types::from((*self.TestField).clone())).unwrap());).to_string()
        );

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new("test_node", "test_field", &optional_definition);

        assert_eq!(
            optional_field.get_children_field().to_string(),
            quote!(if let Some(child) = *self.TestField {
                children.push(&Self::Child::try_from(Types::from((*child).clone())).unwrap());
            })
            .to_string()
        );

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new("test_node", "test_field", &multiple_definition);

        assert_eq!(
            multiple_field.get_children_field().to_string(),
            quote!(
                children.extend(self.TestField.iter().map(|child| &Self::Child::try_from(Types::from((*child).clone())).unwrap()));
            ).to_string()
        );
    }

    #[test]
    fn test_get_children_by_field_name_field() {
        // Test required field
        let field_definition = create_test_field_definition("test_type", false, true);
        let field = Field::new("test_node", "test_field", &field_definition);

        assert_eq!(
            field.get_children_by_field_name_field().to_string(),
            quote!(test_field => vec![&Self::Child::try_from(Types::from((*self.TestField).clone())).unwrap()]).to_string()
        );

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new("test_node", "test_field", &optional_definition);

        assert_eq!(
            optional_field.get_children_by_field_name_field().to_string(),
            quote!(test_field => self.TestField.map_or_else(|| vec![], |child| vec![&Self::Child::try_from(Types::from((*child).clone())).unwrap()])).to_string()
        );

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new("test_node", "test_field", &multiple_definition);

        assert_eq!(
            multiple_field.get_children_by_field_name_field().to_string(),
            quote!(test_field => self.TestField.iter().map(|child| &Self::Child::try_from(Types::from((*child).clone())).unwrap()).collect()).to_string()
        );
    }
}
