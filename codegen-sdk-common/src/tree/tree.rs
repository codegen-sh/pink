use std::hash::Hash;

use indextree::{Arena, NodeId};
use salsa::Update;
pub trait TreeNode: Eq + PartialEq {}
#[derive(Debug, Eq, PartialEq)]
pub struct Tree<T: TreeNode> {
    ids: Arena<T>,
}
impl<T: TreeNode> Hash for Tree<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ids.count().hash(state);
    }
}
impl<T: TreeNode> Default for Tree<T> {
    fn default() -> Self {
        Self { ids: Arena::new() }
    }
}
impl<T: TreeNode> Tree<T> {
    pub fn insert(&mut self, value: T) -> NodeId {
        self.ids.new_node(value)
    }
    pub fn insert_with_children(&mut self, value: T, children: Vec<NodeId>) -> NodeId {
        let id = self.insert(value);
        for child in children {
            id.append(child, &mut self.ids);
        }
        id
    }
    pub fn get(&self, id: &NodeId) -> Option<&T> {
        self.ids.get(*id).map(|node| node.get())
    }
    pub fn descendants(&self, id: &NodeId) -> impl Iterator<Item = (&T, NodeId)> {
        id.descendants(&self.ids)
            .map(|id| (self.get(&id).unwrap(), id))
    }
    pub fn children(&self, id: &NodeId) -> impl Iterator<Item = (&T, NodeId)> {
        id.children(&self.ids)
            .map(|id| (self.get(&id).unwrap(), id))
    }
    pub fn arena(&self) -> &Arena<T> {
        &self.ids
    }
}
unsafe impl<T> Update for Tree<T>
where
    T: TreeNode + Update,
{
    unsafe fn maybe_update(_old_pointer: *mut Self, _new_set: Self) -> bool {
        todo!("Tree is not updateable");
    }
}
