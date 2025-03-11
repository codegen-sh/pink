use std::path::PathBuf;

use crate::Db;

pub trait File<'db> {
    fn path(&self, db: &'db dyn salsa::Database) -> &PathBuf;
    fn content(&'db self, db: &'db dyn Db) -> &'db String {
        let path = self.path(db);
        db.get_file(path).unwrap().content(db)
    }
}
