use crate::{CodebaseContext, ResolveType};

pub trait References<
    'db,
    ReferenceType: ResolveType<'db, Scope, Type = Self> + Clone, // References must resolve to this type
    Scope: crate::Scope<'db, Type = Self, ReferenceType = ReferenceType> + Clone,
>: Eq + PartialEq
{
    fn references<F: TryInto<Scope> + Clone + 'db, T>(&self, codebase: &'db T) -> Vec<ReferenceType>
    where
        Self: Sized,
        for<'b> T: CodebaseContext<File<'db> = F> + 'static,
    {
        let scopes: Vec<Scope> = codebase.files().into_iter().filter_map(|file| file.clone().try_into().ok()).collect();
        return self.references_for_scopes(codebase.db(), scopes);
    }
    fn references_for_scopes(&self, db: &'db dyn salsa::Database, scopes: Vec<Scope>) -> Vec<ReferenceType>
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
