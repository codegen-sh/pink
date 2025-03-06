use crate::{Db, HasFile, Parse, ResolveType};

pub trait References<
    'db,
    ReferenceType: ResolveType<'db, Type = Self> + Clone, // References must resolve to this type
    Scope: crate::Scope<'db, Type = Self, ReferenceType = ReferenceType> + Clone + 'db,
>: Eq + PartialEq + HasFile<'db, File<'db> = Scope> + 'db where Self:'db
{
    fn references(&self, db: &'db dyn Db) -> Vec<ReferenceType>
    where
        Self: Sized,
        Scope: Parse<'db>,
    {
        let files = db.files();
        let root_path = self.root_path(db);
        log::info!("Finding references across {:?} files", files.len());
        let mut results = Vec::new();
        for input in files {
            // if !self.filter(db, &input) {
            //     continue;
            // }
            let file = Scope::parse(db, input, root_path.clone());
            for reference in file.clone().resolvables(db) {
                if reference.clone().resolve_type(db).iter().any(|result| *result == *self) {
                    results.push(reference);
                }
            }
        }
        results
    }
    fn filter(&self, db: &'db dyn Db, input: &codegen_sdk_ast::input::File) -> bool;
}
