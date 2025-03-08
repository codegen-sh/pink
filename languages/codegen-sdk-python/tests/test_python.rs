#![recursion_limit = "512"]
use std::path::PathBuf;

use codegen_sdk_ast::{Definitions, References};
use codegen_sdk_resolution::References as _;
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
fn test_python_ast_class() {
    let temp_dir = tempfile::tempdir().unwrap();
    let content = "
class Test:
    pass";
    let file_path = write_to_temp_file(content, &temp_dir);
    let db = codegen_sdk_cst::CSTDatabase::default();
    let content = codegen_sdk_cst::Input::new(&db, content.to_string());
    let input = codegen_sdk_ast::input::File::new(&db, file_path, content);
    let file = codegen_sdk_python::ast::parse_query(&db, input);
    let tree = file.tree(&db);
    assert_eq!(file.definitions(&db).classes(&db, &tree).len(), 1);
}
#[test_log::test]
fn test_python_ast_function() {
    let temp_dir = tempfile::tempdir().unwrap();
    let content = "
def test():
    pass";
    let file_path = write_to_temp_file(content, &temp_dir);
    let db = codegen_sdk_cst::CSTDatabase::default();
    let content = codegen_sdk_cst::Input::new(&db, content.to_string());
    let input = codegen_sdk_ast::input::File::new(&db, file_path, content);
    let file = codegen_sdk_python::ast::parse_query(&db, input);
    let tree = file.tree(&db);
    assert_eq!(file.definitions(&db).functions(&db, &tree).len(), 1);
}
#[test_log::test]
fn test_python_ast_function_usages() {
    let temp_dir = tempfile::tempdir().unwrap();
    let content = "
def test():
    pass

test()";
    let file_path = write_to_temp_file(content, &temp_dir);
    let db = codegen_sdk_cst::CSTDatabase::default();
    let content = codegen_sdk_cst::Input::new(&db, content.to_string());
    let input = codegen_sdk_ast::input::File::new(&db, file_path, content);
    let file = codegen_sdk_python::ast::parse_query(&db, input);
    let tree = file.tree(&db);
    assert_eq!(file.references(&db).calls(&db, &tree).len(), 1);
    let definitions = file.definitions(&db);
    let functions = definitions.functions(&db, &tree);
    let function = functions.get("test").unwrap().first().unwrap();
    assert_eq!(
        function
            .references_for_scopes(&db, vec![*file], &file)
            .len(),
        1
    );
}
