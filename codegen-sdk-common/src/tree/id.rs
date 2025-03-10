use std::path::PathBuf;

#[salsa::interned(no_lifetime)]
pub struct FileNodeId {
    pub path: PathBuf,
}
#[salsa::interned]
pub struct CSTNodeId<'db> {
    pub file: FileNodeId,
    node_id: usize,
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
