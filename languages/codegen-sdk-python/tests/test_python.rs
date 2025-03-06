#![recursion_limit = "512"]
use std::path::PathBuf;

use codegen_sdk_ast::{Definitions, References};
use codegen_sdk_resolution::References as _;
fn write_to_temp_file(content: &str, temp_dir: &tempfile::TempDir) -> PathBuf {
    write_to_temp_file_with_name(content, temp_dir, "test.py")
}
fn write_to_temp_file_with_name(
    content: &str,
    temp_dir: &tempfile::TempDir,
    name: &str,
) -> PathBuf {
    let file_path = temp_dir.path().join(name);
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
    assert_eq!(file.definitions(&db).classes(&db).len(), 1);
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
    assert_eq!(file.definitions(&db).functions(&db).len(), 1);
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
    assert_eq!(file.references(&db).calls(&db).len(), 1);
    let definitions = file.definitions(&db);
    let functions = definitions.functions(&db);
    let function = functions.get("test").unwrap().first().unwrap();
    let function = codegen_sdk_python::ast::Symbol::Function(function.clone().clone());
    assert_eq!(
        function
            .references_for_scopes(&db, temp_dir.path().to_path_buf(), vec![*file], &file)
            .len(),
        1
    );
}
#[test_log::test]
fn test_python_ast_function_usages_cross_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let content = "
def test():
    pass

";
    let usage_file_content = "
from filea import test
test()";
    let file_path = write_to_temp_file_with_name(content, &temp_dir, "filea.py");
    let usage_file_path = write_to_temp_file_with_name(usage_file_content, &temp_dir, "fileb.py");
    let db = codegen_sdk_cst::CSTDatabase::default();
    let content = codegen_sdk_cst::Input::new(&db, content.to_string());
    let usage_content = codegen_sdk_cst::Input::new(&db, usage_file_content.to_string());
    let input = codegen_sdk_ast::input::File::new(&db, file_path, content);
    let usage_input = codegen_sdk_ast::input::File::new(&db, usage_file_path, usage_content);
    let file = codegen_sdk_python::ast::parse_query(&db, input);
    let usage_file = codegen_sdk_python::ast::parse_query(&db, usage_input);
    assert_eq!(usage_file.references(&db).calls(&db).len(), 1);
    let definitions = file.definitions(&db);
    let functions = definitions.functions(&db);
    let function = functions.get("test").unwrap().first().unwrap();
    let function = codegen_sdk_python::ast::Symbol::Function(function.clone().clone());
    let imports = usage_file.definitions(&db).imports(&db);
    let import = imports.get("test").unwrap().first().unwrap();
    let import = codegen_sdk_python::ast::Symbol::Import(import.clone().clone());
    assert_eq!(
        import
            .references_for_scopes(
                &db,
                temp_dir.path().to_path_buf(),
                vec![*usage_file],
                &usage_file
            )
            .len(),
        1
    );
    assert_eq!(
        function
            .references_for_scopes(
                &db,
                temp_dir.path().to_path_buf(),
                vec![*file, *usage_file],
                &usage_file
            )
            .len(),
        1
    );
}
