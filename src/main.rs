#![recursion_limit = "2048"]
use std::{path::PathBuf, time::Instant};

use clap::Parser;
use codegen_sdk_analyzer::{CodegenDatabase, Db, ParsedFile, parse_file};
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use codegen_sdk_core::{discovery::FilesToParse, parser::parse_files, system::get_memory};
#[derive(Debug, Parser)]
struct Args {
    input: String,
}
fn print_definitions(db: &CodegenDatabase, files_to_parse: &FilesToParse) {
    let mut classes = 0;
    let mut functions = 0;
    let mut interfaces = 0;
    let mut methods = 0;
    let mut modules = 0;
    for file in files_to_parse.files(db) {
        if let Ok(file) = db.input(file) {
            let parsed = parse_file(db, file);
            if let Some(parsed) = parsed.file(db) {
                if let ParsedFile::Typescript(file) = parsed {
                    let definitions = file.definitions(db);
                    classes += definitions.classes.len();
                    functions += definitions.functions.len();
                    interfaces += definitions.interfaces.len();
                    methods += definitions.methods.len();
                    modules += definitions.modules.len();
                }
            }
        }
    }
    log::info!(
        "{} classes, {} functions, {} interfaces, {} methods, {} modules",
        classes,
        functions,
        interfaces,
        methods,
        modules
    );
}
fn main() {
    env_logger::init();
    let args = Args::parse();
    let dir = args.input;
    let start = Instant::now();
    let (tx, rx) = crossbeam_channel::unbounded();
    let mut db = CodegenDatabase::new(tx);
    db.watch_dir(PathBuf::from(&dir)).unwrap();
    let (files_to_parse, errors) = parse_files(
        &db,
        #[cfg(feature = "serialization")]
        &cache,
        dir,
    );
    let num_errors = errors.len();
    drop(errors);
    let end = Instant::now();
    let duration: std::time::Duration = end.duration_since(start);
    let memory = get_memory();
    log::info!(
        "{} files parsed in {:?}.{} seconds with {} errors. Using {} MB of memory",
        files_to_parse.files(&db).len(),
        duration.as_secs(),
        duration.subsec_millis(),
        num_errors,
        memory / 1024 / 1024
    );
    print_definitions(&db, &files_to_parse);
    print_definitions(&db, &files_to_parse);
}
