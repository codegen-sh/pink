use std::hash::Hash;

use crate::{Db, Dependencies, FullyQualifiedName, HasFile, HasId, Parse, ResolveType};

pub trait References<
    'db,
    Dep: Dependencies<'db, FullyQualifiedName, ReferenceType> + 'db,
    ReferenceType: ResolveType<'db, Type = Self> + Eq + Hash + Clone + 'db, // References must resolve to this type
    Scope: crate::Scope<'db, Type = Self, ReferenceType = ReferenceType, Dependencies = Dep> +
    Clone + 'db,
>: Eq + PartialEq + Hash + HasFile<'db, File<'db> = Scope> + HasId<'db> + Sized + 'db where Self:'db
{
    fn references(self, db: &'db dyn Db) -> Vec<ReferenceType>
    where
        Self: Sized,
        Scope: Parse<'db>;
}
