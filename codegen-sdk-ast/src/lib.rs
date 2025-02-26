#![recursion_limit = "512"]
#![allow(unused)]
use codegen_sdk_common::File;
pub use codegen_sdk_common::language::LANGUAGES;
pub use codegen_sdk_cst::*;
use codegen_sdk_macros::include_languages_ast;
pub trait Named {
    fn name(&self) -> &str;
}
impl<T: File> Named for T {
    fn name(&self) -> &str {
        self.path().file_name().unwrap().to_str().unwrap()
    }
}
include_languages_ast!();
