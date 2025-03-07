use std::hash::Hash;

use crate::{Db, HasFile, Parse, ResolveType};

pub trait References<
    'db,
    ReferenceType: ResolveType<'db, Type = Self> + Eq + Hash + Clone + 'db, // References must resolve to this type
    Scope: crate::Scope<'db, Type = Self, ReferenceType = ReferenceType> + Clone + 'db,
>: Eq + PartialEq + Hash + HasFile<'db, File<'db> = Scope> + 'db where Self:'db
{
    fn references(&self, db: &'db dyn Db) -> Vec<ReferenceType>
    where
        Self: Sized,
        Scope: Parse<'db>,
    {
        let files = db.files();
        log::info!("Finding references across {:?} files", files.len());
        let mut results = Vec::new();
        for input in files {
            // if !self.filter(db, &input) {
            //     continue;
            // }
            let file = Scope::parse(db, input);
            let dependencies = file.clone().compute_dependencies_query(db);
            if let Some(references) = dependencies.get(self) {
                results.extend(references.iter().cloned());
            }
        }
        results
    }
    fn filter(&self, db: &'db dyn Db, input: &codegen_sdk_ast::input::File) -> bool;
}
