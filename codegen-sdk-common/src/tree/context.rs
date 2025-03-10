use std::{path::PathBuf, sync::Arc};

use bytes::Bytes;

use crate::tree::{FileNodeId, Tree, TreeNode};
pub struct ParseContext<'db, T: TreeNode> {
    pub db: &'db dyn salsa::Database,
    pub file_id: FileNodeId,
    pub root: FileNodeId,
    pub buffer: Arc<Bytes>,
    pub tree: Tree<T>,
}
impl<'db, T: TreeNode> ParseContext<'db, T> {
    pub fn new(db: &'db dyn salsa::Database, path: PathBuf, root: PathBuf, content: Bytes) -> Self {
        let file_id = FileNodeId::new(db, path);
        Self {
            db,
            file_id,
            root: FileNodeId::new(db, root),
            buffer: Arc::new(content),
            tree: Tree::default(),
        }
    }
}
