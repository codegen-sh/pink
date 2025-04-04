#![recursion_limit = "2048"]
use std::{path::PathBuf, time::Instant};

use clap::Parser;
use codegen_sdk_analyzer::{Codebase, ParsedFile, parse_file};
use codegen_sdk_ast::Definitions;
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use codegen_sdk_core::system::get_memory;
use codegen_sdk_resolution::CodebaseContext;
#[derive(Debug, Parser)]
struct Args {
    input: String,
}
// #[salsa::tracked]
fn get_definitions<'db>(
    db: &'db dyn codegen_sdk_resolution::Db,
    file: codegen_sdk_common::FileNodeId,
) -> (usize, usize, usize, usize, usize, usize) {
    if let Some(parsed) = parse_file(db, file) {
        #[cfg(feature = "typescript")]
        if let ParsedFile::Typescript(file) = parsed {
            let definitions = file.definitions(db);
            return (
                definitions.classes(db).len(),
                definitions.functions(db).len(),
                definitions.interfaces(db).len(),
                definitions.methods(db).len(),
                definitions.modules(db).len(),
                0,
            );
        }
        #[cfg(feature = "python")]
        if let ParsedFile::Python(file) = parsed {
            use codegen_sdk_resolution::References;
            let definitions = file.definitions(db);
            let functions = definitions.functions(db);
            let mut total_references = 0;
            let mut total_functions = 0;
            for function in functions.values().flatten().into_iter() {
                let references = function.references(db);
                total_references += references.len();
                total_functions += 1;
            }
            return (
                definitions.classes(db).values().flatten().count(),
                total_functions,
                0,
                0,
                0,
                total_references,
            );
        }
    }
    (0, 0, 0, 0, 0, 0)
}
fn get_total_definitions(codebase: &Codebase) -> Vec<(usize, usize, usize, usize, usize, usize)> {
    log::info!("Getting total definitions");
    codebase.execute_op_with_progress("Getting Usages", true, |db, file| get_definitions(db, file))
}
fn print_definitions(codebase: &Codebase) {
    let mut total_classes = 0;
    let mut total_functions = 0;
    let mut total_interfaces = 0;
    let mut total_methods = 0;
    let mut total_modules = 0;
    let mut total_references = 0;
    let definitions = get_total_definitions(codebase);
    for (classes, functions, interfaces, methods, modules, references) in definitions {
        total_classes += classes;
        total_functions += functions;
        total_interfaces += interfaces;
        total_methods += methods;
        total_modules += modules;
        total_references += references;
    }
    log::info!(
        "{} classes, {} functions, {} interfaces, {} methods, {} modules, {} references",
        total_classes,
        total_functions,
        total_interfaces,
        total_methods,
        total_modules,
        total_references
    );
}
fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let dir = args.input;
    let start = Instant::now();
    let mut codebase = Codebase::new(PathBuf::from(&dir));
    let end = Instant::now();
    let duration: std::time::Duration = end.duration_since(start);
    let memory = get_memory();
    log::info!(
        "{} files parsed in {:?}.{} seconds with {} errors. Using {} MB of memory",
        codebase.files().len(),
        duration.as_secs(),
        duration.subsec_millis(),
        codebase.errors().len(),
        memory / 1024 / 1024
    );
    loop {
        // Compile the code starting at the provided input, this will read other
        // needed files using the on-demand mechanism.
        print_definitions(&codebase);
        // let diagnostics = compile::accumulated::<Diagnostic>(&db, initial);
        // if diagnostics.is_empty() {
        //     println!("Sum is: {}", sum);
        // } else {
        //     for diagnostic in diagnostics {
        //         println!("{}", diagnostic.0);
        //     }
        // }
        codebase.check_update()?;
        // Wait for file change events, the output can't change unless the
        // inputs change.
    }
}
