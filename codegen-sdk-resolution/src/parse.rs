use crate::Db;

pub trait Parse<'db> {
    fn parse(db: &'db dyn Db, input: codegen_sdk_common::FileNodeId) -> &'db Self;
}
