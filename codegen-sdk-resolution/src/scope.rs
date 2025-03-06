use crate::{Db, ResolveType};
// Resolve a given string name in a scope to a given type
pub trait Scope<'db>: Sized {
    type Type;
    type ReferenceType: ResolveType<'db>;
    fn resolve(self, db: &'db dyn Db, name: String) -> &'db Vec<Self::Type>;
    /// Get all the resolvables (IE: function_calls) in the scope
    fn resolvables(self, db: &'db dyn Db) -> Vec<Self::ReferenceType>;
    fn compute_dependencies(self, db: &'db dyn Db)
    where
        Self: 'db,
    {
        for reference in self.resolvables(db) {
            reference.resolve_type(db);
        }
    }
}
