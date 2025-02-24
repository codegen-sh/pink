use std::num::NonZeroU16;

use crate::{CSTNode, HasChildren, tree::point::Point};
#[derive(Debug, Clone)]
pub struct TreeCursor<'cursor> {
    // Private implementation details
    current: &'cursor dyn CSTNode,
    parents: Vec<&'cursor dyn HasChildren<Child = dyn CSTNode>>,
    field_id: Option<NonZeroU16>,
    exhausted: bool,
}

impl<'cursor> TreeCursor<'cursor> {
    pub fn new<Child, T: CSTNode + HasChildren<Child = Child>>(node: &'cursor T) -> Self {
        Self {
            current: node,
            parents: vec![],
            field_id: None,
            exhausted: false,
        }
    }
    /// Get the tree cursor's current Node.
    pub fn node(&self) -> &'cursor dyn CSTNode {
        self.current
    }

    /// Get the numerical field id of this tree cursor's current node.
    pub fn field_id(&self) -> Option<NonZeroU16> {
        self.field_id
    }

    /// Get the field name of this tree cursor's current node.
    pub fn field_name(&self) -> Option<&'static str> {
        unimplemented!()
    }

    /// Get the depth of the cursor's current node relative to the original node
    /// that the cursor was constructed with.
    pub fn depth(&self) -> usize {
        self.parents.len()
    }

    /// Get the index of the cursor's current node out of all of the descendants
    /// of the original node that the cursor was constructed with
    pub fn descendant_index(&self) -> usize {
        unimplemented!()
    }

    /// Move this cursor to the first child of its current node.
    ///
    /// Returns `true` if the cursor successfully moved, and returns `false`
    /// if there were no children.
    pub fn goto_first_child(&mut self) -> bool {
        let current: &dyn HasChildren<Child = dyn CSTNode> = self.current.try_into().unwrap();
        if let Some(first_child) = &current.first_child() {
            self.parents.push(current);
            self.current = first_child;
            return true;
        }
        false
    }

    /// Move this cursor to the last child of its current node.
    ///
    /// Returns `true` if the cursor successfully moved, and returns `false`
    /// if there were no children.
    pub fn goto_last_child(&mut self) -> bool {
        unimplemented!()
    }

    /// Move this cursor to the parent of its current node.
    ///
    /// Returns `true` if the cursor successfully moved, and returns `false`
    /// if there was no parent node.
    pub fn goto_parent(&mut self) -> bool {
        if let Some(parent) = self.parents.pop() {
            self.current = parent;
            true
        } else {
            false
        }
    }

    /// Move this cursor to the next sibling of its current node.
    ///
    /// Returns `true` if the cursor successfully moved, and returns `false`
    /// if there was no next sibling node.
    pub fn goto_next_sibling(&mut self) -> bool {
        if let Some(parent) = self.parents.last_mut() {
            if let Some(next_sibling) = self.current.next_sibling(parent) {
                self.current = next_sibling;
                return true;
            }
        }
        false
    }

    /// Move this cursor to the previous sibling of its current node.
    ///
    /// Returns `true` if the cursor successfully moved, and returns `false`
    /// if there was no previous sibling node.
    pub fn goto_previous_sibling(&mut self) -> bool {
        unimplemented!()
    }

    /// Move the cursor to the node that is the nth descendant of the original node
    /// that the cursor was constructed with, where zero represents the original node itself.
    pub fn goto_descendant(&mut self, descendant_index: usize) {
        unimplemented!()
    }

    /// Move this cursor to the first child of its current node that contains or
    /// starts after the given byte offset.
    pub fn goto_first_child_for_byte(&mut self, index: usize) -> Option<usize> {
        unimplemented!()
    }

    /// Move this cursor to the first child of its current node that contains or
    /// starts after the given point.
    pub fn goto_first_child_for_point(&mut self, point: Point) -> Option<usize> {
        unimplemented!()
    }

    // /// Re-initialize this tree cursor to start at the original node that the
    // /// cursor was constructed with.
    // pub fn reset<NewChild, NewT: CSTNode + HasChildren<Child = NewChild>>(&mut self, node: &NewT) {
    //     unimplemented!()
    // }

    // /// Re-initialize a tree cursor to the same position as another cursor.
    // pub fn reset_to(&mut self, cursor: &Self) {
    //     unimplemented!()
    // }
}

// Depth-first iterator
impl<'cursor> Iterator for TreeCursor<'cursor> {
    type Item = &'cursor dyn CSTNode;
    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        let ret = Some(self.current);
        if !self.goto_first_child() {
            if !self.goto_next_sibling() {
                while self.goto_parent() {
                    if self.goto_next_sibling() {
                        break; // Found a sibling
                    }
                }
                self.exhausted = true;
            }
        }
        ret
    }
}
