use std::path::PathBuf;

#[salsa::interned(no_lifetime,constructor=_new)]
pub struct FileNodeId {
    #[return_ref]
    pub path: PathBuf,
}
impl FileNodeId {
    // This ensures that the path is canonical - IE: if the path is a symlink, we don't want multiple ids for the same file.
    pub fn new(db: &dyn salsa::Database, path: PathBuf) -> Self {
        Self::_new(db, path.canonicalize().unwrap_or_else(|_| path))
    }
}
#[salsa::interned]
pub struct CSTNodeId<'db> {
    pub file: FileNodeId,
    pub(crate) node_id: usize,
    pub root: FileNodeId,
    // TODO: add a marker for tree-sitter generation
}
#[salsa::interned(no_lifetime)]
pub struct CSTNodeTreeId {
    pub file: FileNodeId,
    node_id: usize,
    pub root: FileNodeId,
    #[return_ref]
    pub id: indextree::NodeId,
    // TODO: add a marker for tree-sitter generation
}
impl CSTNodeTreeId {
    pub fn from_node_id(
        db: &dyn salsa::Database,
        cst_id: &CSTNodeId<'_>,
        node_id: indextree::NodeId,
    ) -> Self {
        Self::new(
            db,
            cst_id.file(db),
            cst_id.node_id(db),
            cst_id.root(db),
            node_id,
        )
    }
}
