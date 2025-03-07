use std::hash::Hash;

use indexmap::{IndexMap, IndexSet};

use crate::{Db, FullyQualifiedName, HasId, ResolveType};
pub trait Dependencies<'db, Type, ReferenceType>: Eq + Hash + Clone {
    fn get(&'db self, db: &'db dyn Db, key: &Type) -> Option<&'db IndexSet<ReferenceType>>;
}
// Resolve a given string name in a scope to a given type
pub trait Scope<'db>: Sized {
    type Type: Eq + Hash + Clone + HasId<'db>;
    type Dependencies: Dependencies<'db, FullyQualifiedName<'db>, Self::ReferenceType>;
    type ReferenceType: ResolveType<'db, Type = Self::Type> + Eq + Hash + Clone;
    fn resolve(self, db: &'db dyn Db, name: String) -> &'db Vec<Self::Type>;
    /// Get all the resolvables (IE: function_calls) in the scope
    fn resolvables(self, db: &'db dyn Db) -> Vec<Self::ReferenceType>;
    fn compute_dependencies_query(self, db: &'db dyn Db) -> &'db Self::Dependencies;
    fn compute_dependencies(
        self,
        db: &'db dyn Db,
    ) -> IndexMap<FullyQualifiedName<'db>, IndexSet<Self::ReferenceType>>
    where
        Self: 'db,
    {
        let mut dependencies: IndexMap<FullyQualifiedName<'db>, IndexSet<Self::ReferenceType>> =
            IndexMap::new();
        for reference in self.resolvables(db) {
            let resolved = reference.clone().resolve_type(db);
            for resolved in resolved {
                dependencies
                    .entry(resolved.fully_qualified_name(db))
                    .or_default()
                    .insert(reference.clone());
            }
        }
        dependencies
    }
}
