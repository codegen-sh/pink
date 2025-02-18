use std::backtrace::Backtrace;

use bytes::Bytes;
use tree_sitter::{self};
use std::sync::Arc;
use crate::{traits::FromNode, ParseError};
pub fn named_children_without_field_names<T: FromNode>(
    node: tree_sitter::Node,
    buffer: &Arc<Bytes>,
) -> Result<Vec<T>, ParseError> {
    let mut children = Vec::new();
    for (index, child) in node.named_children(&mut node.walk()).enumerate() {
        if node.field_name_for_named_child(index as u32).is_none() {
            children.push(T::from_node(child, buffer)?);
        }
    }
    Ok(children)
}

pub fn get_optional_child_by_field_name<T: FromNode>(
    node: &tree_sitter::Node,
    field_name: &str,
    buffer: &Arc<Bytes>,
) -> Result<Option<T>, ParseError> {
    if let Some(child) = node.child_by_field_name(field_name) {
        return Ok(Some(T::from_node(child, buffer)?));
    }
    Ok(None)
}
pub fn get_child_by_field_name<T: FromNode>(
    node: &tree_sitter::Node,
    field_name: &str,
    buffer: &Arc<Bytes>,
) -> Result<T, ParseError> {
    if let Some(child) = get_optional_child_by_field_name(node, field_name, buffer)? {
        return Ok(child);
    }
    Err(ParseError::MissingNode {
        field_name: field_name.to_string(),
        parent_node: node.kind().to_string(),
        backtrace: Backtrace::capture(),
    })
}

pub fn get_multiple_children_by_field_name<T: FromNode>(
    node: &tree_sitter::Node,
    field_name: &str,
    buffer: &Arc<Bytes>,
) -> Result<Vec<T>, ParseError> {
    let mut children = Vec::new();
    for child in node.children_by_field_name(field_name, &mut node.walk()) {
        children.push(T::from_node(child, buffer)?);
    }
    Ok(children)
}
