#![recursion_limit = "512"]
use std::{env, path::PathBuf};

use codegen_sdk_ast::{Definitions, References};
use codegen_sdk_resolution::References as _;
fn write_to_temp_file_with_name(
    content: &str,
    temp_dir: &tempfile::TempDir,
    name: &str,
) -> PathBuf {
    let file_path = temp_dir.path().join(name);
    std::fs::write(&file_path, content).unwrap();
    file_path
}
fn parse_file<'db>(
    db: &'db dyn codegen_sdk_resolution::Db,
    content: &str,
    temp_dir: &tempfile::TempDir,
    name: &str,
) -> &'db codegen_sdk_python::ast::PythonFile<'db> {
    let file_path = write_to_temp_file_with_name(content, temp_dir, name);
    db.input(&file_path).unwrap();
    let file_node_id = codegen_sdk_common::FileNodeId::new(db, file_path);
    let file = codegen_sdk_python::ast::parse(db, file_node_id);
    file
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
    let db = codegen_sdk_cst::CSTDatabase::default();
    let file = parse_file(&db, content, &temp_dir, "filea.py");
    assert_eq!(file.definitions(&db).classes(&db).len(), 1);
}
#[test_log::test]
fn test_python_ast_function() {
    let temp_dir = tempfile::tempdir().unwrap();
    let content = "
def test():
    pass";
    let db = codegen_sdk_cst::CSTDatabase::default();
    let file = parse_file(&db, content, &temp_dir, "filea.py");
    assert_eq!(file.definitions(&db).functions(&db).len(), 1);
}
//
// for function in codebase.functions():
//     function.rename("test2")
//     codebase.commit()

// 3 bounds
// 1. Codebase updated, everything else is invalidated
// 2. Files + codebase updated, everything else is invalidated
// 3. Everything updated, nothing is invalidated

#[test_log::test]
fn test_python_ast_function_usages() {
    let temp_dir = tempfile::tempdir().unwrap();
    assert!(env::set_current_dir(&temp_dir).is_ok());
    let content = "
def test():
    pass

test()";
    let db = codegen_sdk_cst::CSTDatabase::default();
    let file = parse_file(&db, content, &temp_dir, "filea.py");
    assert_eq!(file.references(&db).calls(&db).len(), 1);
    let definitions = file.definitions(&db);
    let functions = definitions.functions(&db);
    let function = functions.get("test").unwrap().first().unwrap();
    let function = codegen_sdk_python::ast::Symbol::Function(function.clone().clone());
    assert_eq!(function.references(&db).len(), 1);
}
#[test_log::test]
fn test_python_ast_function_usages_cross_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    assert!(env::set_current_dir(&temp_dir).is_ok());
    let content = "
def test():
    pass

";
    let usage_file_content = "
from filea import test
test()";
    let db = codegen_sdk_cst::CSTDatabase::default();
    let file = parse_file(&db, content, &temp_dir, "filea.py");
    let usage_file = parse_file(&db, usage_file_content, &temp_dir, "fileb.py");
    assert_eq!(usage_file.references(&db).calls(&db).len(), 1);
    let definitions = file.definitions(&db);
    let functions = definitions.functions(&db);
    let function = functions.get("test").unwrap().first().unwrap();
    let function = codegen_sdk_python::ast::Symbol::Function(function.clone().clone());
    let imports = usage_file.definitions(&db).imports(&db);
    let import = imports.get("test").unwrap().first().unwrap();
    let import = codegen_sdk_python::ast::Symbol::Import(import.clone().clone());
    assert_eq!(import.references(&db,).len(), 1);
    assert_eq!(function.references(&db,).len(), 1);
}
