#![recursion_limit = "512"]
use codegen_sdk_common::File;
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
#[cfg(test)]
mod tests {
    use codegen_sdk_common::Language;
    use codegen_sdk_cst::ts_query::TypescriptFile;

    use super::*;
    #[test]
    fn test_typescript_ast() {
        let file = TypescriptFile::new(PathBuf::from("test.ts"));
        let ast = file.parse();
        assert_eq!(ast.is_ok(), true);
    }
}
