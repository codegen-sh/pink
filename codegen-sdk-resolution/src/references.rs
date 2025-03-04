use crate::ResolveType;

pub trait References<
    'db,
    ReferenceType: ResolveType<'db, Scope, Type = Self> + Clone, // References must resolve to this type
    Scope: crate::Scope<'db, Type = Self, ReferenceType = ReferenceType> + Clone,
>: Eq + PartialEq
{
    fn references(&self, db: &'db dyn salsa::Database, scopes: Vec<Scope>) -> Vec<ReferenceType>
    where
        Self: Sized,
    {
        let mut results = Vec::new();
        for scope in scopes {
            for reference in scope.clone().resolvables(db) {
                let resolved = reference.clone().resolve_type(db, scope.clone());
                if resolved.iter().any(|result| result == self) {
                    results.push(reference);
                }
            }
        }
        results
    }
}
