use std::{fmt::Debug, sync::Arc};

use bytes::Bytes;
use tree_sitter::{self};

use crate::{errors::ParseError, point::Point};
pub trait FromNode: Sized {
    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError>;
}
pub trait CSTNode: Send + Debug {
    fn start_byte(&self) -> usize;
    fn end_byte(&self) -> usize;
    fn start_position(&self) -> Point;
    fn end_position(&self) -> Point;
    fn buffer(&self) -> &Bytes;
    fn text(&self) -> Bytes {
        Bytes::copy_from_slice(&self.buffer()[self.start_byte()..self.end_byte()])
    }
    fn source(&self) -> String {
        String::from_utf8(self.text().to_vec()).unwrap()
    }
    fn kind_id(&self) -> u16;
    fn kind(&self) -> &str;

}
pub trait HasNode: Send + Debug {
    type Node: CSTNode + HasChildren;
    fn node(&self) -> &Self::Node;
}
impl<T: HasNode> CSTNode for T {
    fn kind(&self) -> &str {
        self.node().kind()
    }
    fn start_byte(&self) -> usize {
        self.node().start_byte()
    }
    fn end_byte(&self) -> usize {
        self.node().end_byte()
    }
    fn start_position(&self) -> Point {
        self.node().start_position()
    }
    fn end_position(&self) -> Point {
        self.node().end_position()
    }
    fn buffer(&self) -> &Bytes {
        self.node().buffer()
    }
    fn kind_id(&self) -> u16 {
        self.node().kind_id()
    }
    
}
impl<T: HasNode> HasChildren for T {
    type Child = <T::Node as HasChildren>::Child;
    fn child_by_field_name(&self, field_name: &str) -> Option<&Self::Child> {
        self.node().child_by_field_name(field_name)
    }
    fn children_by_field_name(&self, field_name: &str) -> Vec<&Self::Child> {
        self.node().children_by_field_name(field_name)
    }
    fn children(&self) -> Vec<&Self::Child> {
        self.node().children()
    }
}
pub trait HasChildren {
    type Child: Send;
    fn child_by_field_name(&self, field_name: &str) -> Option<&Self::Child> {
        self.children_by_field_name(field_name)
            .first()
            .map(|child| *child)
    }
    fn children_by_field_name(&self, field_name: &str) -> Vec<&Self::Child>;
    fn children(&self) -> Vec<&Self::Child>;
}
