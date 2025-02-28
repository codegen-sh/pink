use std::{ops::Div, path, time::Instant};

use clap::Parser;
use codegen_sdk_analyzer::{CodegenDatabase, Db, Parsed};
use codegen_sdk_ast::*;
use codegen_sdk_common::serialize::Cache;
use glob::glob;
use rayon::prelude::*;
use sysinfo::System;
fn report_cached_count(cached: usize, files_to_parse: &Vec<path::PathBuf>) {
    log::info!(
        "{} files cached. {}% of total",
        cached,
        (cached * 100).div(files_to_parse.len())
    );
}
fn get_cached_count(cache: &Cache, files_to_parse: &Vec<path::PathBuf>) -> usize {
    let mut cached = 0;
    for file in files_to_parse.iter() {
        let path = cache.get_path(file);
        if path.exists() {
            cached += 1;
        }
    }
    cached
}
