use std::{collections::HashMap, path::PathBuf, sync::Arc};

use bytes::Bytes;

use crate::tree::{CSTNodeId, FileNodeId};
pub struct ParseContext<'db, T> {
    pub db: &'db dyn salsa::Database,
    pub file_id: FileNodeId<'db>,
    pub buffer: Arc<Bytes>,
    ids: HashMap<CSTNodeId<'db>, T>,
}
impl<'db, T> ParseContext<'db, T> {
    pub fn insert(&mut self, id: CSTNodeId<'db>, value: T) {
        self.ids.insert(id, value);
    }
    pub fn new(db: &'db dyn salsa::Database, path: PathBuf, content: Bytes) -> Self {
        let file_id = FileNodeId::new(db, path);
        Self {
            db,
            file_id,
            buffer: Arc::new(content),
            ids: HashMap::new(),
        }
    }
}
