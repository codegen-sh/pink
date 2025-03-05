#![recursion_limit = "512"]
use std::path::PathBuf;

use codegen_sdk_ast::Definitions;
fn write_to_temp_file(content: &str, temp_dir: &tempfile::TempDir) -> PathBuf {
    let file_path = temp_dir.path().join("test.ts");
    std::fs::write(&file_path, content).unwrap();
    file_path
}
// TODO: Fix queries for classes and functions
// #[test_log::test]
// fn test_typescript_ast_class() {
//     let temp_dir = tempfile::tempdir().unwrap();
//     let content = "class Test { }";
//     let file_path = write_to_temp_file(content, &temp_dir);
//     let file = TypescriptFile::parse(&file_path).unwrap();
//     assert_eq!(file.visitor.classes.len(), 1);
// }
// #[test_log::test]
// fn test_typescript_ast_function() {
//     let temp_dir = tempfile::tempdir().unwrap();
//     let content = "function test() { }";
//     let file_path = write_to_temp_file(content, &temp_dir);
//     let file = TypescriptFile::parse(&file_path).unwrap();
//     assert_eq!(file.visitor.functions.len(), 1);
// }
#[test_log::test]
fn test_typescript_ast_interface() {
    let temp_dir = tempfile::tempdir().unwrap();
    let content = "interface Test { }";
    let file_path = write_to_temp_file(content, &temp_dir);
    let db = codegen_sdk_cst::CSTDatabase::default();
    let content = codegen_sdk_cst::Input::new(&db, content.to_string());
    let input = codegen_sdk_ast::input::File::new(&db, file_path, content);
    let file = codegen_sdk_typescript::ast::parse_query(&db, input);
    let tree = file.node(&db).unwrap().tree(&db);
    assert_eq!(file.definitions(&db).interfaces(db, &tree).len(), 1);
}
