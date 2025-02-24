use std::{fmt::Debug, sync::Arc};

use bytes::Bytes;
use tree_sitter::{self};

use crate::{Point, errors::ParseError, tree::Range};
pub trait FromNode: Sized {
    fn from_node(node: tree_sitter::Node, buffer: &Arc<Bytes>) -> Result<Self, ParseError>;
}
#[enum_delegate::register]
pub trait CSTNode {
    /// Returns the byte offset where the node starts
    fn start_byte(&self) -> usize;

    /// Returns the byte offset where the node ends
    fn end_byte(&self) -> usize;

    /// Returns the position where the node starts
    fn start_position(&self) -> Point;

    /// Returns the position where the node ends
    fn end_position(&self) -> Point;

    /// Returns the range of positions that this node spans
    fn range(&self) -> Range {
        Range::new(self.start_position(), self.end_position())
    }

    /// Returns the source text buffer for this node
    fn buffer(&self) -> &Bytes;

    /// Returns the raw text content of this node as bytes
    fn text(&self) -> Bytes {
        Bytes::copy_from_slice(&self.buffer()[self.start_byte()..self.end_byte()])
    }

    /// Returns the text content of this node as a String
    fn source(&self) -> std::string::String {
        String::from_utf8(self.text().to_vec()).unwrap()
    }
    /// Returns the node's type as a numerical id
    fn kind_id(&self) -> u16;

    /// Returns the node's type as a string
    fn kind(&self) -> &str;

    /// Returns true if this node is named, false if it is anonymous
    fn is_named(&self) -> bool;

    /// Returns true if this node represents a syntax error
    fn is_error(&self) -> bool {
        unimplemented!("is_error not implemented")
    }

    /// Returns true if this node is *missing* from the source code
    fn is_missing(&self) -> bool {
        unimplemented!("is_missing not implemented")
    }

    /// Returns true if this node has been edited
    fn is_edited(&self) -> bool {
        unimplemented!("is_edited not implemented")
    }

    /// Returns true if this node represents extra tokens from the source code
    fn is_extra(&self) -> bool {
        unimplemented!("is_extra not implemented")
    }
    fn id(&self) -> usize;
}
trait CSTNodeExt: CSTNode {
    /// Get the next sibling of this node in its parent
    fn next_sibling<Child: CSTNode + Clone, Parent: HasChildren<Child = Child>>(
        &self,
        parent: &Parent,
    ) -> Option<Child> {
        let mut iter = parent.children().into_iter();
        while let Some(child) = iter.next() {
            if child.id() == self.id() {
                return iter.next();
            }
        }
        None
    }
    fn next_named_sibling<Child: CSTNode + Clone, Parent: HasChildren<Child = Child>>(
        &self,
        parent: &Parent,
    ) -> Option<Child> {
        let mut iter = parent.named_children().into_iter();
        while let Some(child) = iter.next() {
            if child.id() == self.id() {
                return iter.next();
            }
        }
        None
    }
    fn prev_sibling<Child: CSTNode + Clone, Parent: HasChildren<Child = Child>>(
        &self,
        parent: &Parent,
    ) -> Option<Child> {
        let mut prev = None;
        for child in parent.children() {
            if child.id() == self.id() {
                return prev;
            }
            prev = Some(child);
        }
        None
    }
    fn prev_named_sibling<Child: CSTNode + Clone, Parent: HasChildren<Child = Child>>(
        &self,
        parent: &Parent,
    ) -> Option<Child> {
        let mut prev = None;
        for child in parent.named_children() {
            if child.id() == self.id() {
                return prev;
            }
            prev = Some(child);
        }
        None
    }
}
pub trait HasNode: Send + Debug + Clone {
    type Node: CSTNode;
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
    fn is_named(&self) -> bool {
        self.node().is_named()
    }
    fn is_error(&self) -> bool {
        self.node().is_error()
    }
    fn is_missing(&self) -> bool {
        self.node().is_missing()
    }
    fn is_edited(&self) -> bool {
        self.node().is_edited()
    }
    fn is_extra(&self) -> bool {
        self.node().is_extra()
    }

    fn id(&self) -> usize {
        self.node().id()
    }
}
// impl<T: HasNode> HasChildren for T {
//     type Child = <T::Node as HasChildren>::Child;
//     fn child_by_field_name(&self, field_name: &str) -> Option<Self::Child> {
//         self.node().child_by_field_name(field_name)
//     }
//     fn children_by_field_name(&self, field_name: &str) -> Vec<Self::Child> {
//         self.node().children_by_field_name(field_name)
//     }
//     fn children(&self) -> Vec<Self::Child> {
//         self.node().children()
//     }
//     fn child_by_field_id(&self, field_id: u16) -> Option<Self::Child> {
//         self.node().child_by_field_id(field_id)
//     }
//     fn child_count(&self) -> usize {
//         self.node().child_count()
//     }
// }
pub trait HasChildren {
    type Child: Send + Debug + Clone + CSTNode;
    /// Returns the first child with the given field name
    fn child_by_field_id(&self, field_id: u16) -> Option<Self::Child> {
        self.children_by_field_id(field_id)
            .first()
            .map(|child| child.clone())
    }

    /// Returns all children with the given field name
    fn children_by_field_id(&self, _field_id: u16) -> Vec<Self::Child>;

    /// Returns the first child with the given field name
    fn child_by_field_name(&self, field_name: &str) -> Option<Self::Child> {
        self.children_by_field_name(field_name)
            .first()
            .map(|child| child.clone())
    }

    /// Returns all children with the given field name
    fn children_by_field_name(&self, field_name: &str) -> Vec<Self::Child>;

    /// Returns all children of the node
    fn children(&self) -> Vec<Self::Child>;
    /// Returns all named children of the node
    fn named_children(&self) -> Vec<Self::Child> {
        self.children()
            .into_iter()
            .filter(|child| child.is_named())
            .collect()
    }

    // /// Returns a cursor for walking the tree starting from this node
    // fn walk(&self) -> TreeCursor
    // where
    //     Self: Sized,
    // {
    //     TreeCursor::new(self)
    // }

    /// Returns the first child of the node
    fn first_child(&self) -> Option<Self::Child> {
        self.children().into_iter().next()
    }

    /// Returns the last child of the node
    fn last_child(&self) -> Option<Self::Child> {
        self.children().into_iter().last()
    }
    /// Returns the number of children of this node
    fn child_count(&self) -> usize {
        self.children().len()
    }
}
