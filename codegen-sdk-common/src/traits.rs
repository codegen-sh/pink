use std::{fmt::Debug, sync::Arc};

use bytes::Bytes;
use delegation::delegate;
use tree_sitter::{self};

use crate::{Point, errors::ParseError, tree::Range};
pub trait FromNode<'db>: Sized {
    fn from_node(
        db: &'db dyn salsa::Database,
        node: tree_sitter::Node,
        buffer: &Arc<Bytes>,
    ) -> Result<Self, ParseError>;
}
#[delegate]
pub trait CSTNode<'db: 'db> {
    /// Returns the byte offset where the node starts
    fn start_byte(self, db: &'db dyn salsa::Database) -> usize;

    /// Returns the byte offset where the node ends
    fn end_byte(self, db: &'db dyn salsa::Database) -> usize;

    /// Returns the position where the node starts
    fn start_position(self, db: &'db dyn salsa::Database) -> Point<'db>;

    /// Returns the position where the node ends
    fn end_position(self, db: &'db dyn salsa::Database) -> Point<'db>;

    /// Returns the source text buffer for this node
    fn buffer(self, db: &'db dyn salsa::Database) -> &'db Bytes;

    /// Returns the node's type as a numerical id
    fn kind_id(self, db: &'db dyn salsa::Database) -> u16;

    /// Returns the node's type as a string
    fn kind(self, db: &'db dyn salsa::Database) -> &'db String;

    /// Returns true if this node is named, false if it is anonymous
    fn is_named(self, db: &'db dyn salsa::Database) -> bool;

    /// Returns true if this node represents a syntax error
    fn is_error(self, db: &'db dyn salsa::Database) -> bool;

    fn id(self, db: &'db dyn salsa::Database) -> usize;
}
pub trait CSTNodeExt<'db>: CSTNode<'db> + Clone {
    /// Get the next sibling of this node in its parent
    fn next_sibling<Child: CSTNode<'db> + Clone, Parent: HasChildren<'db, Child = Child>>(
        self,
        db: &'db dyn salsa::Database,
        parent: Parent,
    ) -> Option<Child>
    where
        Self: Sized,
    {
        let id = self.id(db);
        let mut iter = parent.children(db).into_iter();
        while let Some(child) = iter.next() {
            if child.id(db) == id {
                return iter.next();
            }
        }
        None
    }
    fn next_named_sibling<Child: CSTNode<'db> + Clone, Parent: HasChildren<'db, Child = Child>>(
        self,
        db: &'db dyn salsa::Database,
        parent: Parent,
    ) -> Option<Child>
    where
        Self: Sized,
    {
        let id = self.id(db);
        let mut iter = parent.named_children(db).into_iter();
        while let Some(child) = iter.next() {
            if child.id(db) == id {
                return iter.next();
            }
        }
        None
    }
    fn prev_sibling<Child: CSTNode<'db> + Clone, Parent: HasChildren<'db, Child = Child>>(
        self,
        db: &'db dyn salsa::Database,
        parent: Parent,
    ) -> Option<Child>
    where
        Self: Sized,
    {
        let id = self.id(db);
        let mut prev = None;
        for child in parent.children(db).into_iter() {
            if child.clone().id(db) == id {
                return prev;
            }
            prev = Some(child);
        }
        None
    }
    fn prev_named_sibling<Child: CSTNode<'db> + Clone, Parent: HasChildren<'db, Child = Child>>(
        self,
        db: &'db dyn salsa::Database,
        parent: Parent,
    ) -> Option<Child>
    where
        Self: Sized,
    {
        let id = self.id(db);
        let mut prev = None;
        for child in parent.named_children(db) {
            if child.clone().id(db) == id {
                return prev;
            }
            prev = Some(child);
        }
        None
    }
    /// Returns the raw text content of this node as bytes
    fn text(&self, db: &'db dyn salsa::Database) -> Bytes
    where
        Self: Sized,
    {
        let start_byte = self.clone().start_byte(db);
        let end_byte = self.clone().end_byte(db);
        let buffer = self.clone().buffer(db);
        Bytes::copy_from_slice(&buffer[start_byte..end_byte])
    }

    /// Returns the text content of this node as a String
    fn source(&self, db: &'db dyn salsa::Database) -> std::string::String
    where
        Self: Sized,
    {
        String::from_utf8(self.text(db).to_vec()).unwrap()
    }
    /// Returns the range of positions that this node spans
    fn range(self, db: &'db dyn salsa::Database) -> Range<'db>
    where
        Self: Sized,
    {
        let start = self.clone().start_position(db);
        let end = self.clone().end_position(db);
        Range::from_points(db, start, end)
    }
    /// Returns true if this node is *missing* from the source code
    fn is_missing(&self, db: &'db dyn salsa::Database) -> bool {
        unimplemented!("is_missing not implemented")
    }

    /// Returns true if this node has been edited
    fn is_edited(&self, db: &'db dyn salsa::Database) -> bool {
        unimplemented!("is_edited not implemented")
    }

    /// Returns true if this node represents extra tokens from the source code
    fn is_extra(&self, db: &'db dyn salsa::Database) -> bool {
        unimplemented!("is_extra not implemented")
    }
}
// pub trait HasNode<'db>: Send + Debug + Clone {
//     type Node: CSTNode<'db> + Clone;
//     fn node(&self) -> &Self::Node;
//     fn node_owned(self) -> Self::Node {
//         self.node().clone()
//     }
// }
// impl<'db, T: HasNode<'db>> CSTNode<'db> for T {
//     fn kind(self, db: &'db dyn salsa::Database) -> &'db String {
//         self.node_owned().kind(db)
//     }
//     fn start_byte(self, db: &'db dyn salsa::Database) -> usize {
//         self.node_owned().start_byte(db)
//     }
//     fn end_byte(self, db: &'db dyn salsa::Database) -> usize {
//         self.node_owned().end_byte(db)
//     }
//     fn start_position(self, db: &'db dyn salsa::Database) -> Point<'db> {
//         self.node_owned().start_position(db)
//     }
//     fn end_position(self, db: &'db dyn salsa::Database) -> Point<'db> {
//         self.node_owned().end_position(db)
//     }
//     fn buffer(self, db: &'db dyn salsa::Database) -> &'db Bytes {
//         self.node_owned().buffer(db)
//     }
//     fn kind_id(self, db: &'db dyn salsa::Database) -> u16 {
//         self.node_owned().kind_id(db)
//     }
//     fn is_named(self, db: &'db dyn salsa::Database) -> bool {
//         self.node_owned().is_named(db)
//     }
//     fn is_error(self, db: &'db dyn salsa::Database) -> bool {
//         self.node_owned().is_error(db)
//     }
//     // fn is_missing(&self, db: &'db dyn salsa::Database) -> bool {
//     //     self.node().is_missing(db)
//     // }
//     // fn is_edited(&self, db: &'db dyn salsa::Database) -> bool {
//     //     self.node().is_edited(db)
//     // }
//     // fn is_extra(&self, db: &'db dyn salsa::Database) -> bool {
//     //     self.node().is_extra(db)
//     // }

//     fn id(self, db: &'db dyn salsa::Database) -> usize {
//         self.node_owned().id(db)
//     }
// }
// // impl<T: HasNode> HasChildren for T {
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
pub trait HasChildren<'db>: Sized {
    type Child: Send + Debug + Clone + CSTNode<'db>;
    /// Returns the first child with the given field name
    fn child_by_field_id(self, db: &'db dyn salsa::Database, field_id: u16) -> Option<Self::Child> {
        self.children_by_field_id(db, field_id)
            .first()
            .map(|child| child.clone())
    }

    /// Returns all children with the given field name
    fn children_by_field_id(self, db: &'db dyn salsa::Database, field_id: u16) -> Vec<Self::Child>;

    /// Returns the first child with the given field name
    fn child_by_field_name(
        self,
        db: &'db dyn salsa::Database,
        field_name: &str,
    ) -> Option<Self::Child> {
        self.children_by_field_name(db, field_name)
            .first()
            .map(|child| child.clone())
    }

    /// Returns all children with the given field name
    fn children_by_field_name(
        self,
        db: &'db dyn salsa::Database,
        field_name: &str,
    ) -> Vec<Self::Child>;

    /// Returns all children of the node
    fn children(self, db: &'db dyn salsa::Database) -> Vec<Self::Child>;
    /// Returns all named children of the node
    fn named_children(self, db: &'db dyn salsa::Database) -> Vec<Self::Child> {
        self.children(db)
            .into_iter()
            .filter_map(|child| {
                if child.clone().is_named(db) {
                    Some(child)
                } else {
                    None
                }
            })
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
    fn first_child(self, db: &'db dyn salsa::Database) -> Option<Self::Child> {
        self.children(db).into_iter().next()
    }

    /// Returns the last child of the node
    fn last_child(self, db: &'db dyn salsa::Database) -> Option<Self::Child> {
        self.children(db).into_iter().last()
    }
    /// Returns the number of children of this node
    fn child_count(self, db: &'db dyn salsa::Database) -> usize {
        self.children(db).len()
    }
}
