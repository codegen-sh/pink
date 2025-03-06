use std::path::PathBuf;

use crate::Scope;
// Get definitions for a given type
pub trait ResolveType<'db, T: Scope<'db>> {
    type Type; // Possible types this trait can be defined as
    fn resolve_type(
        self,
        db: &'db dyn salsa::Database,
        scope: T,
        root_path: PathBuf,
        scopes: Vec<T>,
    ) -> &'db Vec<Self::Type>;
}
