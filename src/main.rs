use std::{path::PathBuf, time::Instant};

use clap::Parser;
use codegen_sdk_analyzer::{CodegenDatabase, Db};
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use codegen_sdk_core::{parser::parse_files, system::get_memory};
#[derive(Debug, Parser)]
struct Args {
    input: String,
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
}
