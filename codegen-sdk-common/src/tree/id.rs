use std::path::PathBuf;

#[salsa::interned(no_lifetime)]
pub struct FileNodeId {
    #[return_ref]
    pub path: PathBuf,
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
