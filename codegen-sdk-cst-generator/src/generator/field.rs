#[double]
use codegen_sdk_common::language::Language;
use codegen_sdk_common::{
    naming::{normalize_field_name, normalize_type_name},
    parser::{FieldDefinition, TypeDefinition},
};
use mockall_double::double;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::constants::TYPE_NAME;
use crate::Config;

#[derive(Debug)]
pub struct Field<'a> {
    raw: &'a FieldDefinition,
    name: String,
    node_name: String,
    language: &'a Language,
    config: Config,
}

impl<'a> Field<'a> {
    pub fn new(
        node_name: &str,
        name: &str,
        raw: &'a FieldDefinition,
        language: &'a Language,
        config: Config,
    ) -> Self {
        Self {
            node_name: node_name.to_string(),
            name: name.to_string(),
            raw,
            language,
            config,
        }
    }
    fn field_id(&self) -> u16 {
        self.language.field_id(&self.name).unwrap().into()
    }
    pub fn name(&self) -> String {
        normalize_field_name(&self.name)
    }
    pub fn normalized_name(&self) -> String {
        normalize_type_name(&self.name, true)
    }
    pub fn types(&self) -> Vec<&TypeDefinition> {
        self.raw.types.iter().collect()
    }
    pub fn type_name(&self) -> String {
        let types = self.types();
        if types.len() == 1 {
            normalize_type_name(&types[0].type_name, types[0].named)
        } else {
            format!("{}{}", self.node_name, self.normalized_name())
        }
    }
    pub fn get_constructor_field(&self) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.name());
        let original_name = &self.name;
        if self.raw.multiple {
            quote! {
                #field_name_ident: get_multiple_children_by_field_name(db, &node, #original_name, buffer)?
            }
        } else if !self.raw.required {
            quote! {
                #field_name_ident: Box::new(get_optional_child_by_field_name(db, &node, #original_name, buffer)?)
            }
        } else {
            quote! {
                #field_name_ident: Box::new(get_child_by_field_name(db,&node, #original_name, buffer)?)
            }
        }
    }
    pub fn get_convert_child(&self, convert_children: bool) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.name());
        let types = format_ident!("{}", TYPE_NAME);
        if convert_children {
            if self.raw.multiple {
                quote! {
                    Self::Child::try_from(#types::from(child.clone())).unwrap()
                }
            } else if !self.raw.required {
                quote! {
                    Self::Child::try_from(#types::from(child.clone())).unwrap()
                }
            } else {
                quote! {
                    Self::Child::try_from(#types::from(self.#field_name_ident.as_ref().clone())).unwrap()
                }
            }
        } else if self.raw.multiple || !self.raw.required {
            quote! {
                child.clone()
            }
        } else {
            quote! {
                self.#field_name_ident.as_ref().clone()
            }
        }
    }
    pub fn get_children_field(&self, convert_children: bool) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.name());
        let convert_child = self.get_convert_child(convert_children);

        if self.raw.multiple {
            quote! {
                children.extend(self.#field_name_ident.iter().map(|child| #convert_child));
            }
        } else if self.raw.required {
            quote! {
                children.push(#convert_child);
            }
        } else {
            quote! {
                if let Some(child) = self.#field_name_ident.as_ref() {
                    children.push(#convert_child);
                }
            }
        }
    }
    pub fn get_children_by_field_name_field(&self, convert_children: bool) -> TokenStream {
        let field_name = &self.name;
        let field_name_ident = format_ident!("{}", self.name());
        let convert_child = self.get_convert_child(convert_children);

        if self.raw.multiple {
            quote! {
                #field_name => self.#field_name_ident.iter().map(|child| #convert_child).collect()
            }
        } else if self.raw.required {
            quote! {
                #field_name => vec![#convert_child]
            }
        } else {
            quote! {
                #field_name => self.#field_name_ident.as_ref().iter().map(|child| #convert_child).collect()
            }
        }
    }
    pub fn get_children_by_field_id_field(&self, convert_children: bool) -> TokenStream {
        let field_id = self.field_id();
        let field_name_ident = format_ident!("{}", self.name());
        let convert_child = self.get_convert_child(convert_children);

        if self.raw.multiple {
            quote! {
                #field_id => self.#field_name_ident.iter().map(|child| #convert_child).collect()
            }
        } else if self.raw.required {
            quote! {
                #field_id => vec![#convert_child]
            }
        } else {
            quote! {
                #field_id => self.#field_name_ident.as_ref().iter().map(|child| #convert_child).collect()
            }
        }
    }
    pub fn get_struct_field(&self) -> TokenStream {
        let field_name_ident = format_ident!("{}", self.name());
        let converted_type_name = format_ident!("{}", self.type_name());
        let bounds = if self.config.serialize {
            quote! {
                #[rkyv(omit_bounds)]
            }
        } else {
            quote! {}
        };
        if self.raw.multiple {
            quote! {
                #bounds
                pub #field_name_ident: Vec<#converted_type_name<'db>>
            }
        } else if !self.raw.required {
            quote! {
                #bounds
                pub #field_name_ident: Box<Option<#converted_type_name<'db>>>
            }
        } else {
            quote! {
                #bounds
                pub #field_name_ident: Box<#converted_type_name<'db>>
            }
        }
    }
    pub fn is_optional(&self) -> bool {
        !self.raw.required
    }
    pub fn is_multiple(&self) -> bool {
        self.raw.multiple
    }
}

#[cfg(test)]
mod tests {
    use codegen_sdk_common::parser::TypeDefinition;

    use super::*;
    use crate::test_util::{get_language_no_nodes, snapshot_tokens};
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
        let language = get_language_no_nodes();
        let field = Field::new(
            "node",
            "field",
            &field_definition,
            &language,
            Config::default(),
        );
        assert_eq!(field.normalized_name(), "Field");
    }

    #[test]
    fn test_field_types() {
        let field_definition = create_test_field_definition_variants(
            &vec!["type_a".to_string(), "type_b".to_string()],
            false,
            true,
        );
        let language = get_language_no_nodes();
        let field = Field::new(
            "test_node",
            "test_field",
            &field_definition,
            &language,
            Config::default(),
        );
        assert_eq!(
            field.types(),
            field_definition.types.iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_field_type_name() {
        let field_definition = create_test_field_definition_variants(
            &vec!["test_type".to_string(), "test_type".to_string()],
            false,
            true,
        );
        let language = get_language_no_nodes();
        let field = Field::new(
            "Node",
            "field",
            &field_definition,
            &language,
            Config::default(),
        );
        assert_eq!(field.type_name(), "NodeField");
    }

    #[test]
    fn test_get_struct_field() {
        let field_definition = create_test_field_definition("test_type", false, true);
        let language = get_language_no_nodes();
        let field = Field::new(
            "test_node",
            "test_field",
            &field_definition,
            &language,
            Config::default(),
        );
        insta::assert_debug_snapshot!(snapshot_tokens(&field.get_struct_field()));

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new(
            "test_node",
            "test_field",
            &optional_definition,
            &language,
            Config::default(),
        );
        insta::assert_debug_snapshot!(snapshot_tokens(&optional_field.get_struct_field()));

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new(
            "test_node",
            "test_field",
            &multiple_definition,
            &language,
            Config::default(),
        );
        insta::assert_debug_snapshot!(snapshot_tokens(&multiple_field.get_struct_field()));
    }

    #[test]
    fn test_get_constructor_field() {
        let field_definition = create_test_field_definition("test_type", false, true);
        let language = get_language_no_nodes();
        let field = Field::new(
            "test_node",
            "test_field",
            &field_definition,
            &language,
            Config::default(),
        );
        insta::assert_debug_snapshot!(snapshot_tokens(&field.get_constructor_field()));

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new(
            "test_node",
            "test_field",
            &optional_definition,
            &language,
            Config::default(),
        );
        insta::assert_debug_snapshot!(snapshot_tokens(&optional_field.get_constructor_field()));

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new(
            "test_node",
            "test_field",
            &multiple_definition,
            &language,
            Config::default(),
        );
        insta::assert_debug_snapshot!(snapshot_tokens(&multiple_field.get_constructor_field()));
    }

    #[test]
    fn test_get_children_field() {
        let field_definition = create_test_field_definition("test_type", false, true);
        let language = get_language_no_nodes();
        let field = Field::new(
            "test_node",
            "test_field",
            &field_definition,
            &language,
            Config::default(),
        );

        assert_eq!(
            field.get_children_field(true).to_string(),
            quote!(children.push(Self::Child::try_from(NodeTypes::from(self.test_field.as_ref().clone())).unwrap());).to_string()
        );

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new(
            "test_node",
            "test_field",
            &optional_definition,
            &language,
            Config::default(),
        );

        assert_eq!(
            optional_field.get_children_field(true).to_string(),
            quote!(if let Some(child) = self.test_field.as_ref() {
                children.push(Self::Child::try_from(NodeTypes::from(child.clone())).unwrap());
            })
            .to_string()
        );

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new(
            "test_node",
            "test_field",
            &multiple_definition,
            &language,
            Config::default(),
        );

        assert_eq!(
            multiple_field.get_children_field(true).to_string(),
            quote!(children.extend(self.test_field.iter().map(|child| Self::Child::try_from(NodeTypes::from(child.clone())).unwrap()));).to_string()
        );
    }

    #[test]
    fn test_get_children_by_field_name_field() {
        let field_definition = create_test_field_definition("test_type", false, true);
        let language = get_language_no_nodes();
        let field = Field::new(
            "test_node",
            "test_field",
            &field_definition,
            &language,
            Config::default(),
        );

        assert_eq!(
            field.get_children_by_field_name_field(true).to_string(),
            quote!("test_field" => vec![Self::Child::try_from(NodeTypes::from(self.test_field.as_ref().clone())).unwrap()]).to_string()
        );

        // Test optional field
        let optional_definition = create_test_field_definition("test_type", false, false);
        let optional_field = Field::new(
            "test_node",
            "test_field",
            &optional_definition,
            &language,
            Config::default(),
        );

        assert_eq!(
            optional_field.get_children_by_field_name_field(true).to_string(),
            quote!("test_field" => self.test_field.as_ref().iter().map(|child| Self::Child::try_from(NodeTypes::from(child.clone())).unwrap()).collect()).to_string()
        );

        // Test multiple field
        let multiple_definition = create_test_field_definition("test_type", true, true);
        let multiple_field = Field::new(
            "test_node",
            "test_field",
            &multiple_definition,
            &language,
            Config::default(),
        );

        assert_eq!(
            multiple_field.get_children_by_field_name_field(true).to_string(),
            quote!("test_field" => self.test_field.iter().map(|child| Self::Child::try_from(NodeTypes::from(child.clone())).unwrap()).collect()).to_string()
        );
    }
}
