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
    // {
    //     // let files = files(db);
    //     // log::info!(target: "resolution", "Finding references across {:?} files", files.len());
    //     // let mut results = Vec::new();
    //     // for input in files {
    //     //     // if !self.filter(db, &input) {
    //     //     //     continue;
    //     //     // }
    //     //     let file = Scope::parse(db, input.clone());
    //     //     let dependencies = file.clone().compute_dependencies_query(db);
    //     //     if let Some(references) = dependencies.get(db, &self.fully_qualified_name(db)) {
    //     //         results.extend(references.iter().cloned());
    //     //     }
    //     // }
    //     results
    // }
    fn filter(&self, db: &'db dyn Db, input: &codegen_sdk_cst::File) -> bool;
}
