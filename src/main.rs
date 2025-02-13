use clap::Parser;
use codegen_sdk_cst::{parse_file_typescript, tsx};
use glob::glob;
use rayon::prelude::*;
use std::{path, time::Instant};
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
) -> Option<Box<tsx::Program>> {
    if file.is_dir() {
        return None;
    }
    return match parse_file_typescript(file.to_str().unwrap()) {
        Ok(program) => Some(program),
        Err(e) => {
            tx.send(e.to_string()).unwrap();
            None
        }
    };
}
fn parse_files(dir: String) -> (Vec<Box<tsx::Program>>, Vec<String>) {
    rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 1024 * 10)
        .build_global()
        .unwrap();
    let (tx, rx) = crossbeam::channel::unbounded();
    let mut errors = Vec::new();
    let files_to_parse = collect_files(dir);
    log::info!("Parsing {} files", files_to_parse.len());
    let files: Vec<Box<tsx::Program>> = files_to_parse
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
