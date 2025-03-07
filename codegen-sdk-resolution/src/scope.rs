use std::hash::Hash;

use indexmap::IndexMap;

use crate::{Db, ResolveType};
// Resolve a given string name in a scope to a given type
pub trait Scope<'db>: Sized {
    type Type: Eq + Hash + Clone;
    type ReferenceType: ResolveType<'db, Type = Self::Type> + Eq + Hash + Clone;
    fn resolve(self, db: &'db dyn Db, name: String) -> &'db Vec<Self::Type>;
    /// Get all the resolvables (IE: function_calls) in the scope
    fn resolvables(self, db: &'db dyn Db) -> Vec<Self::ReferenceType>;
    fn compute_dependencies_query(
        self,
        db: &'db dyn Db,
    ) -> &'db IndexMap<Self::Type, Vec<Self::ReferenceType>>;
    fn compute_dependencies(self, db: &'db dyn Db) -> IndexMap<Self::Type, Vec<Self::ReferenceType>>
    where
        Self: 'db,
    {
        let mut dependencies: IndexMap<Self::Type, Vec<Self::ReferenceType>> = IndexMap::new();
        for reference in self.resolvables(db) {
            let resolved = reference.clone().resolve_type(db);
            for resolved in resolved {
                dependencies
                    .entry(resolved.clone())
                    .or_default()
                    .push(reference.clone());
            }
        }
        dependencies
    }
}
