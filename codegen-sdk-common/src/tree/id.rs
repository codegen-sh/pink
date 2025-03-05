use std::path::PathBuf;

#[salsa::interned]
pub struct FileNodeId<'db> {
    file_path: PathBuf,
}
#[salsa::interned]
pub struct CSTNodeId<'db> {
    file_id: FileNodeId<'db>,
    node_id: usize,
    // TODO: add a marker for tree-sitter generation
}
