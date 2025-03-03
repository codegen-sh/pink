#![recursion_limit = "512"]
pub mod input;
use ambassador::delegatable_trait;
use codegen_sdk_common::File;
pub use codegen_sdk_common::language::LANGUAGES;
pub use codegen_sdk_cst::*;
pub trait Named {
    fn name(&self) -> &str;
}
impl<T: File> Named for T {
    fn name(&self) -> &str {
        self.path().file_name().unwrap().to_str().unwrap()
    }
}
#[delegatable_trait]
pub trait Definitions<'db> {
    type Definitions;
    fn definitions(self, db: &'db dyn salsa::Database) -> Self::Definitions;
}
#[delegatable_trait]
pub trait References<'db> {
    type References;
    fn references(self, db: &'db dyn salsa::Database) -> Self::References;
}
#[delegatable_trait]
pub trait FileExt<'db>: References<'db> + Definitions<'db> + Clone {
    fn precompute(self, db: &'db dyn salsa::Database)
    where
        Self: Sized,
    {
        self.clone().definitions(db);
        self.references(db);
    }
}
