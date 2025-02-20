use std::{path, time::Instant};

use clap::Parser;
use codegen_sdk_common::{language::LANGUAGES, serialize::Cache, traits::CSTNode};
use glob::glob;
use rayon::prelude::*;
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
    let mut files = Vec::new();
    for language in LANGUAGES.iter() {
        for extension in language.file_extensions.iter() {
            files.extend(glob(&format!("{dir}**/*.{}", extension)).unwrap());
        }
    }

    files.into_iter().filter_map(|file| file.ok()).collect()
}
fn parse_file(
    cache: &Cache,
    file: &path::PathBuf,
    tx: &crossbeam::channel::Sender<String>,
) -> Option<Box<dyn CSTNode + Send>> {
    if file.is_dir() {
        return None;
    }
    let result = codegen_sdk_cst::parse_file(cache, file);

    return match result {
        Ok(program) => Some(program),
        Err(e) => {
            log::error!("Error parsing file {}: {}", file.display(), e);
            tx.send(e.to_string()).unwrap();
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
    log_languages();
    let cache = Cache::new().unwrap();
    let files_to_parse = collect_files(dir);
    log::info!("Parsing {} files", files_to_parse.len());
    let mut cached = 0;
    for file in files_to_parse.iter() {
        let path = cache.get_path(file);
        if path.exists() {
            cached += 1;
        }
    }
    let files: Vec<Box<dyn CSTNode + Send>> = files_to_parse
        .par_iter()
        .filter_map(|file| parse_file(&cache, file, &tx))
        .collect();
    drop(tx);
    for e in rx.iter() {
        errors.push(e);
    }
    log::info!(
        "{} files cached. {}% of total",
        cached,
        cached * 100 / files_to_parse.len()
    );
    (files, errors)
}
fn main() {
    env_logger::init();
    let args = Args::parse();
    let dir = args.input;
    let start = Instant::now();
    let (files, errors) = parse_files(dir);
    let num_errors = errors.len();
    drop(errors);
    let end = Instant::now();
    let duration: std::time::Duration = end.duration_since(start);
    let memory = get_memory();
    log::info!(
        "{} files parsed in {:?}.{} seconds with {} errors. Using {} MB of memory",
        files.len(),
        duration.as_secs(),
        duration.subsec_millis(),
        num_errors,
        memory / 1024 / 1024
    );
}
