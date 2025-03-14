use crate::Db;

pub trait Compute<'db> {
    fn compute(db: &'db dyn Db);
}
