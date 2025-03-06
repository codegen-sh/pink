use std::path::PathBuf;

#[salsa::interned]
pub struct FileNodeId<'db> {
    pub path: PathBuf,
}
#[salsa::interned]
pub struct CSTNodeId<'db> {
    pub file: FileNodeId<'db>,
    node_id: usize,
    pub root: FileNodeId<'db>,
    // TODO: add a marker for tree-sitter generation
}
