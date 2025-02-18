use crate::generator::state::State;
use codegen_sdk_common::naming::{normalize_string, normalize_type_name};
use codegen_sdk_common::parser::TypeDefinition;
fn get_cases(
    variants: &Vec<TypeDefinition>,
    cases: &mut String,
    state: &State,
    override_variant_name: Option<&str>,
    existing_cases: &mut Vec<String>,
) {
    for t in variants {
        let normalized_variant_name = normalize_type_name(&t.type_name);
        if normalized_variant_name.is_empty() {
            continue;
        }
        let variant_name = override_variant_name.unwrap_or_else(|| &normalized_variant_name);
        let prefix = format!("{}::{}", "Self", variant_name);
        if let Some(variants) = state.variants.get(&normalized_variant_name) {
            get_cases(variants, cases, state, Some(variant_name), existing_cases);
        } else if !existing_cases.contains(&t.type_name) {
            existing_cases.push(t.type_name.clone());
            cases.push_str(&format!(
                "\"{}\" => Ok({}({variant_name}::from_node(node, buffer)?)),",
                t.type_name, prefix,
            ));
        }
    }
}
pub fn generate_enum(
    variants: &Vec<TypeDefinition>,
    state: &mut State,
    enum_name: &str,
    anonymous_nodes: bool,
) {
    state.enums.push_str(&format!(
        "
    #[derive(Debug, Clone, Archive, Portable, Deserialize, Serialize)]
    #[repr(C, u8)]
    pub enum {enum_name} {{\n",
        enum_name = enum_name
    ));
    for t in variants {
        let variant_name = normalize_type_name(&t.type_name);
        if variant_name.is_empty() {
            continue;
        }
        state
            .enums
            .push_str(&format!("    {}({variant_name}),\n", variant_name));
    }
    if anonymous_nodes {
        state.enums.push_str("    Anonymous,\n");
    }
    state.enums.push_str("}\n");
    let mut cases = String::new();
    let mut existing_cases = Vec::new();
    get_cases(variants, &mut cases, state, None, &mut existing_cases);
    if anonymous_nodes {
        for (name, _variant_name) in state.anonymous_nodes.iter() {
            if name.is_empty() {
                continue;
            }
            if existing_cases.contains(name) {
                continue;
            }
            let normalized_name = normalize_string(name);
            cases.push_str(&format!(
                "\"{}\" => Ok(Self::Anonymous),\n",
                normalized_name
            ));
        }
    }
    state.enums.push_str(&format!(
        "
    impl FromNode for {enum_name} {{
        fn from_node(node: tree_sitter::Node, buffer: &Bytes) -> Result<Self, ParseError> {{
            match node.kind() {{
                {cases}
                _ => Err(ParseError::UnexpectedNode {{
                    node_type: node.kind().to_string(),
                    backtrace: Backtrace::capture(),
                }}),
            }}
        }}
    }}
    ",
        enum_name = enum_name,
        cases = cases
    ));
}
