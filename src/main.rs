use clap::Parser;
use codegen_sdk_common::{language::LANGUAGES, traits::CSTNode};
use glob::glob;
use rayon::prelude::*;
use std::{panic::catch_unwind, path, time::Instant};
use sysinfo::System;
#[derive(Debug, Parser)]
struct Args {
    input: String,
}
fn get_memory() -> u64 {
    let s = System::new_all();
    let current = s.process(sysinfo::get_current_pid().unwrap()).unwrap();
    current.memory()
}
fn collect_files(dir: String) -> Vec<path::PathBuf> {
    glob(&format!("{}/**/*.ts*", dir))
        .unwrap()
        .filter_map(|file| file.ok())
        .collect()
}
fn parse_file(
    file: &path::PathBuf,
    tx: &crossbeam::channel::Sender<String>,
) -> Option<Box<dyn CSTNode + Send>> {
    if file.is_dir() {
        return None;
    }
    let result = catch_unwind(|| codegen_sdk_cst::parse_file(file));

    return match result {
        Ok(Ok(program)) => Some(program),
        Ok(Err(e)) => {
            tx.send(e.to_string()).unwrap();
            None
        }
        Err(_) => {
            tx.send("".to_string()).unwrap();
            None
        }
    };
}
fn log_languages() {
    for language in LANGUAGES.iter() {
        log::info!(
            "Supported language: {} with extensions: {:?}",
            language.name,
            language.file_extensions
        );
    }
}
fn parse_files(dir: String) -> (Vec<Box<dyn CSTNode + Send>>, Vec<String>) {
    rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 1024 * 10)
        .build_global()
        .unwrap();
    let (tx, rx) = crossbeam::channel::unbounded();
    let mut errors = Vec::new();
    let files_to_parse = collect_files(dir);
    log::info!("Parsing {} files", files_to_parse.len());
    log_languages();
    let files: Vec<Box<dyn CSTNode + Send>> = files_to_parse
        .par_iter()
        .filter_map(|file| parse_file(file, &tx))
        .collect();
    drop(tx);
    for e in rx.iter() {
        errors.push(e);
    }
    (files, errors)
}
fn main() {
    env_logger::init();
    let args = Args::parse();
    let dir = args.input;
    let start = Instant::now();
    let (files, errors) = parse_files(dir);
    let end = Instant::now();
    let duration: std::time::Duration = end.duration_since(start);
    let memory = get_memory();
    log::info!(
        "{} files parsed in {:?}.{} seconds with {} errors. Using {} MB of memory",
        files.len(),
        duration.as_secs(),
        duration.subsec_millis(),
        errors.len(),
        memory / 1024 / 1024
    );
}
