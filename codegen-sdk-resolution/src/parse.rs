use salsa::Database;

pub trait Parse<'db> {
    fn parse(db: &'db dyn Database, input: codegen_sdk_cst::File) -> &'db Self;
}
