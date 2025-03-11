mod scope;
use std::path::PathBuf;

use ambassador::delegatable_trait;
pub use scope::Scope;
mod resolve_type;
pub use resolve_type::ResolveType;
mod references;
pub use references::References;
mod codebase;
pub use codebase::CodebaseContext;
mod database;
mod parse;
pub use database::{Db, files};
pub use parse::Parse;
pub use scope::Dependencies;
mod file;
mod name;
pub use file::File;
pub use name::{FullyQualifiedName, HasId};
#[delegatable_trait]
pub trait HasFile<'db> {
    type File<'db1>: File<'db1>;
    fn file(&self, db: &'db dyn Db) -> &'db Self::File<'db>;
    fn root_path(&self, db: &'db dyn Db) -> &PathBuf;
}
