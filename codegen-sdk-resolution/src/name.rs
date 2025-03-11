use codegen_sdk_common::FileNodeId;

#[salsa::interned(no_lifetime)]
pub struct FullyQualifiedName {
    #[id]
    pub file: FileNodeId,
    #[return_ref]
    pub name: String,
}

pub trait HasId<'db> {
    fn fully_qualified_name(&self, db: &'db dyn salsa::Database) -> FullyQualifiedName;
}
