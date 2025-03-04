use std::path::PathBuf;

use salsa::Database;
// Not sure what to name this
// Equivalent to CodebaseGraph/CodebaseContext in the SDK
pub trait CodebaseContext {
    type File<'a>
    where
        Self: 'a;
    fn files<'a>(&'a self) -> Vec<&'a Self::File<'a>>;
    fn db(&self) -> &dyn Database;
    fn get_file<'a>(&'a self, path: PathBuf) -> Option<&'a Self::File<'a>>;
}
