use crate::errors::ParseError;
use bytes::Bytes;
use tree_sitter::{self, Point};
pub trait FromNode: Sized {
    fn from_node(node: tree_sitter::Node) -> Result<Self, ParseError>;
}
pub trait CSTNode: Send {
    fn start_byte(&self) -> usize;
    fn end_byte(&self) -> usize;
    fn start_position(&self) -> Point;
    fn end_position(&self) -> Point;
    fn text(&self) -> &Bytes;
    fn source(&self) -> String {
        String::from_utf8(self.text().to_vec()).unwrap()
    }
}
pub trait HasChildren {
    type Child: Send;
    fn children(&self) -> &Vec<Self::Child>;
}
