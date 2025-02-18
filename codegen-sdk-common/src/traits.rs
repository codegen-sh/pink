use std::fmt::Debug;

use crate::errors::ParseError;
use crate::point::Point;
use bytes::Bytes;
use tree_sitter::{self};
pub trait FromNode: Sized {
    fn from_node(node: tree_sitter::Node, buffer: &Bytes) -> Result<Self, ParseError>;
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
}
pub trait HasNode: Send + Debug {
    type Node: CSTNode;
    fn node(&self) -> &Self::Node;
}
impl<T: HasNode> CSTNode for T {
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
pub trait HasChildren {
    type Child: Send;
    fn children(&self) -> &Vec<Self::Child>;
}
