use codegen_sdk_common::FileNodeId;

use crate::Db;
#[salsa::interned]
pub struct FullyQualifiedName<'db> {
    #[id]
    pub path: FileNodeId<'db>,
    #[return_ref]
    pub name: String,
}

pub trait HasId<'db> {
    fn fully_qualified_name(&self, db: &'db dyn salsa::Database) -> FullyQualifiedName<'db>;
}
