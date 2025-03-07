use crate::Db;
// Get definitions for a given type
pub trait ResolveType<'db> {
    type Type; // Possible types this trait can be defined as
    fn resolve_type(self, db: &'db dyn Db) -> &'db Vec<Self::Type>;
}
