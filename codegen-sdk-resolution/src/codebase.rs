use std::path::PathBuf;

use salsa::Database;
// Not sure what to name this
// Equivalent to CodebaseGraph/CodebaseContext in the SDK
pub trait CodebaseContext {
    type File<'a>
    where
        Self: 'a;
    fn files(&self) -> Vec<&Self::File<'_>>;
    fn db(&self) -> &dyn Database;
    fn get_file(&self, path: PathBuf) -> Option<&Self::File<'_>>;
}
