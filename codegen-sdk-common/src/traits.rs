use std::fmt::Debug;

use ambassador::delegatable_trait;
use bytes::Bytes;
use indextree::NodeId;
use tree_sitter::{self};

use crate::{
    Point, Tree,
    errors::ParseError,
    tree::{CSTNodeId, FileNodeId, ParseContext, TreeNode},
};
pub trait FromNode<'db, Types: TreeNode>: Sized {
    fn from_node(
        context: &mut ParseContext<'db, Types>,
        node: tree_sitter::Node,
    ) -> Result<(Self, Vec<NodeId>), ParseError>;
    fn orphaned(
        context: &mut ParseContext<'db, Types>,
        node: tree_sitter::Node,
    ) -> Result<NodeId, ParseError>
    where
        Self: Into<Types>,
        Types: CSTNode<'db>,
    {
        let (raw, mut children) = Self::from_node(context, node)?;
        children.sort_by_key(|id| context.tree.get(id).unwrap().start_byte());
        let id = context.tree.insert_with_children(raw.into(), children);
        Ok(id)
    }
}
#[delegatable_trait]
pub trait CSTNode<'db>
where
    Self: 'db,
{
    /// Returns the byte offset where the node starts
    fn start_byte(&self) -> usize;

    /// Returns the byte offset where the node ends
    fn end_byte(&self) -> usize;

    /// Returns the position where the node starts
    fn start_position(&self) -> Point<'db>;

    /// Returns the position where the node ends
    fn end_position(&self) -> Point<'db>;

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
    fn kind_name(&self) -> &str;

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
    fn id(&self) -> CSTNodeId<'db>;
    fn file_id(&self) -> FileNodeId;
}

// pub trait CSTNodeExt<'db>: CSTNode<'db> {
//     /// Get the next sibling of this node in its parent
//     fn next_sibling<Child: CSTNode<'db> + Clone, Parent: HasChildren<'db, Child<'db> = Child>>(
//         &self,
//         parent: &'db Parent,
//         tree: &'db dyn Tree,
//     ) -> Option<Child> {
//         let mut iter = parent.children().into_iter();
//         while let Some(child) = iter.next() {
//             if child.id() == self.id() {
//                 return iter.next();
//             }
//         }
//         None
//     }
//     fn next_named_sibling<
//         Child: CSTNode<'db> + Clone,
//         Parent: HasChildren<'db, Child<'db> = Child>,
//     >(
//         &self,
//         parent: &'db Parent,
//     ) -> Option<Child> {
//         let mut iter = parent.named_children().into_iter();
//         while let Some(child) = iter.next() {
//             if child.id() == self.id() {
//                 return iter.next();
//             }
//         }
//         None
//     }
//     fn prev_sibling<Child: CSTNode<'db> + Clone, Parent: HasChildren<'db, Child<'db> = Child>>(
//         &self,
//         parent: &'db Parent,
//     ) -> Option<Child> {
//         let mut prev = None;
//         for child in parent.children() {
//             if child.id() == self.id() {
//                 return prev;
//             }
//             prev = Some(child);
//         }
//         None
//     }
//     fn prev_named_sibling<
//         Child: CSTNode<'db> + Clone,
//         Parent: HasChildren<'db, Child<'db> = Child>,
//     >(
//         &self,
//         parent: &'db Parent,
//     ) -> Option<Child> {
//         let mut prev = None;
//         for child in parent.named_children() {
//             if child.id() == self.id() {
//                 return prev;
//             }
//             prev = Some(child);
//         }
//         None
//     }
//     /// Returns the range of positions that this node spans
//     fn range(&self, db: &'db dyn salsa::Database) -> Range<'db> {
//         Range::from_points(db, self.start_position(), self.end_position())
//     }
// }
// pub trait HasNode<'db>: Send + Debug + Clone {
//     type Node: CSTNode<'db>;
//     fn node(&self) -> &Self::Node;
// }
// impl<'db, T: HasNode<'db>> CSTNode<'db> for T {
//     fn kind(&self) -> &'_ str {
//         self.node().kind()
//     }
//     fn start_byte(&self) -> usize {
//         self.node().start_byte()
//     }
//     fn end_byte(&self) -> usize {
//         self.node().end_byte()
//     }
//     fn start_position(&self) -> Point<'db> {
//         self.node().start_position()
//     }
//     fn end_position(&self) -> Point<'db> {
//         self.node().end_position()
//     }
//     fn buffer(&self) -> &'_ Bytes {
//         self.node().buffer()
//     }
//     fn kind_id(&self) -> u16 {
//         self.node().kind_id()
//     }
//     fn is_named(&self) -> bool {
//         self.node().is_named()
//     }
//     fn is_error(&self) -> bool {
//         self.node().is_error()
//     }
//     fn is_missing(&self) -> bool {
//         self.node().is_missing()
//     }
//     fn is_edited(&self) -> bool {
//         self.node().is_edited()
//     }
//     fn is_extra(&self) -> bool {
//         self.node().is_extra()
//     }

//     fn id(&self) -> usize {
//         self.node().id()
//     }
// }
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
pub trait HasChildren<'db, Types: TreeNode> {
    type Child<'db2>: Send + Debug
    where
        Self: 'db2,
        Types: 'db2,
        'db: 'db2;
    /// Returns the first child with the given field name
    fn child_by_field_id<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_id: u16,
    ) -> Option<Self::Child<'db1>>
    where
        Self::Child<'db1>: Clone,
    {
        self.children_by_field_id(context, field_id)
            .first()
            .map(|child| child.clone())
    }

    /// Returns all children with the given field name
    fn children_by_field_id<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_id: u16,
    ) -> Vec<Self::Child<'db1>>;

    /// Returns the first child with the given field name
    fn child_by_field_name<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_name: &str,
    ) -> Option<Self::Child<'db1>>
    where
        Self::Child<'db1>: Clone,
    {
        self.children_by_field_name(context, field_name)
            .first()
            .map(|child| child.clone())
    }

    /// Returns all children with the given field name
    fn children_by_field_name<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_name: &str,
    ) -> Vec<Self::Child<'db1>>;

    /// Returns all children of the node
    fn children<'db1>(&'db1 self, context: &'db1 Tree<Types>) -> Vec<Self::Child<'db1>>;
    /// Returns all named children of the node
    fn named_children<'db1>(&'db1 self, context: &'db1 Tree<Types>) -> Vec<Self::Child<'db1>>
    where
        Self::Child<'db1>: CSTNode<'db1>,
    {
        self.children(context)
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
    fn first_child<'db1>(&'db1 self, context: &'db1 Tree<Types>) -> Option<Self::Child<'db1>> {
        self.children(context).into_iter().next()
    }

    /// Returns the last child of the node
    fn last_child<'db1>(&'db1 self, context: &'db1 Tree<Types>) -> Option<Self::Child<'db1>> {
        self.children(context).into_iter().last()
    }
    /// Returns the number of children of this node
    fn child_count(&'db self, context: &'db Tree<Types>) -> usize {
        self.children(context).len()
    }
    fn children_by_field_types<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_types: &[&str],
    ) -> Vec<Self::Child<'db1>>
    where
        Self::Child<'db1>: CSTNode<'db1>,
    {
        self.children(context)
            .into_iter()
            .filter(|child| field_types.contains(&child.kind_name()))
            .collect()
    }
    fn children_by_field_type<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_type: &str,
    ) -> Vec<Self::Child<'db1>>
    where
        Self::Child<'db1>: CSTNode<'db1>,
    {
        self.children_by_field_types(context, &[field_type])
    }
    fn child_by_field_type<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_type: &str,
    ) -> Option<Self::Child<'db1>>
    where
        Self::Child<'db1>: CSTNode<'db1>,
    {
        self.children_by_field_type(context, field_type)
            .into_iter()
            .next()
    }
    fn child_by_field_types<'db1>(
        &'db1 self,
        context: &'db1 Tree<Types>,
        field_types: &[&str],
    ) -> Option<Self::Child<'db1>>
    where
        Self::Child<'db1>: CSTNode<'db1>,
    {
        self.children_by_field_types(context, field_types)
            .into_iter()
            .next()
    }
}
