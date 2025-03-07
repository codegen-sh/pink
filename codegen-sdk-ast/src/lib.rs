#![recursion_limit = "512"]
use ambassador::delegatable_trait;
// use codegen_sdk_common::File;
pub use codegen_sdk_common::language::LANGUAGES;
pub use codegen_sdk_cst::*;
// pub trait Named {
//     fn name(&self) -> &str;
// }
// impl<T: File> Named for T {
//     fn name(&self) -> &str {
//         self.path().file_name().unwrap().to_str().unwrap()
//     }
// }
#[delegatable_trait]
pub trait Definitions<'db> {
    type Definitions;
    fn definitions(self, db: &'db dyn salsa::Database) -> &'db Self::Definitions;
}
#[delegatable_trait]
pub trait References<'db> {
    type References;
    fn references(self, db: &'db dyn salsa::Database) -> &'db Self::References;
}
