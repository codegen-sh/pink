#![recursion_limit = "2048"]
use std::{path::PathBuf, time::Instant};

use anyhow::Context;
use clap::Parser;
use codegen_sdk_analyzer::{CodegenDatabase, Db, ParsedFile, parse_file};
use codegen_sdk_ast::Input;
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use codegen_sdk_core::{discovery::FilesToParse, parser::parse_files, system::get_memory};
use salsa::Setter;
#[derive(Debug, Parser)]
struct Args {
    input: String,
}
#[salsa::tracked]
fn get_total_definitions(
    db: &dyn Db,
    files_to_parse: FilesToParse,
) -> Vec<(usize, usize, usize, usize, usize)> {
    salsa::par_map(db, files_to_parse.files(db), |db, file| {
        let parsed = parse_file(db, file);
        if let Some(parsed) = parsed.file(db) {
            if let ParsedFile::Typescript(file) = parsed {
                let definitions = file.definitions(db);
                return (
                    definitions.classes.len(),
                    definitions.functions.len(),
                    definitions.interfaces.len(),
                    definitions.methods.len(),
                    definitions.modules.len(),
                );
            }
        }
        (0, 0, 0, 0, 0)
    })
}
#[cfg(feature = "typescript")]
fn print_definitions(db: &CodegenDatabase, files_to_parse: &FilesToParse) {
    let mut total_classes = 0;
    let mut total_functions = 0;
    let mut total_interfaces = 0;
    let mut total_methods = 0;
    let mut total_modules = 0;
    let new_files = FilesToParse::new(db, files_to_parse.files(db).clone());
    let definitions = get_total_definitions(db, new_files);
    for (classes, functions, interfaces, methods, modules) in definitions {
        total_classes += classes;
        total_functions += functions;
        total_interfaces += interfaces;
        total_methods += methods;
        total_modules += modules;
    }
    log::info!(
        "{} classes, {} functions, {} interfaces, {} methods, {} modules",
        total_classes,
        total_functions,
        total_interfaces,
        total_methods,
        total_modules
    );
}
fn main() -> anyhow::Result<()> {
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
    loop {
        // Compile the code starting at the provided input, this will read other
        // needed files using the on-demand mechanism.
        print_definitions(&db, &files_to_parse);
        // let diagnostics = compile::accumulated::<Diagnostic>(&db, initial);
        // if diagnostics.is_empty() {
        //     println!("Sum is: {}", sum);
        // } else {
        //     for diagnostic in diagnostics {
        //         println!("{}", diagnostic.0);
        //     }
        // }

        // Wait for file change events, the output can't change unless the
        // inputs change.
        for event in rx.recv()?.unwrap() {
            match event.path.canonicalize() {
                Ok(path) => {
                    log::info!("File changed: {}", path.display());
                    let file = match db.files.get(&path) {
                        Some(file) => *file,
                        None => continue,
                    };
                    // `path` has changed, so read it and update the contents to match.
                    // This creates a new revision and causes the incremental algorithm
                    // to kick in, just like any other update to a salsa input.
                    let contents = std::fs::read_to_string(path)
                        .with_context(|| format!("Failed to read file {}", event.path.display()))?;
                    let input = Input::new(&db, contents);
                    file.set_contents(&mut db).to(input);
                }
                Err(e) => {
                    log::error!(
                        "Failed to canonicalize path {} for file {}",
                        e,
                        event.path.display()
                    );
                }
            }
        }
    }
}
