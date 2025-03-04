use crate::ResolveType;

// Resolve a given string name in a scope to a given type
pub trait Scope<'db>: Sized {
    type Type;
    type ReferenceType: ResolveType<'db, Self, Type = Self::Type>;
    fn resolve(self, db: &'db dyn salsa::Database, name: String) -> Vec<Self::Type>;
    /// Get all the resolvables (IE: function_calls) in the scope
    fn resolvables(self, db: &'db dyn salsa::Database) -> Vec<Self::ReferenceType>;
}
