use std::path;

use codegen_sdk_analyzer::{CodegenDatabase, Db, Parsed};
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use indicatif::ProgressBar;

use crate::discovery::{FilesToParse, collect_files, log_languages};
fn parse_file<'db>(
    db: &'db dyn Db,
    #[cfg(feature = "serialization")] cache: &Cache,
    file: &path::PathBuf,
) -> Parsed<'db> {
    if file.is_dir() {
        log::warn!("Skipping directory: {}", file.display());
        return Parsed::new(db, None);
    }
    let result = db.input(file.into());
    return match result {
        Ok(program) => codegen_sdk_analyzer::parse_file(db, program),
        Err(e) => {
            log::error!("Error parsing file {}: {}", file.display(), e);
            Parsed::new(db, None)
        }
    };
}
#[salsa::tracked]
fn parse_files_par(db: &dyn Db, files: FilesToParse) {
    let multi = db.multi_progress();
    let pg = multi.add(ProgressBar::new(files.files(db).len() as u64));
    let inputs = files
        .files(db)
        .into_iter()
        .map(|file| (&pg, file))
        .collect::<Vec<_>>();
    let _: Vec<()> = salsa::par_map(db, inputs, |db, input| {
        let (pg, file) = input;
        parse_file(
            db,
            #[cfg(feature = "serialization")]
            &cache,
            &file,
        );
        pg.inc(1);
        ()
    });
    pg.finish();
    multi.remove(&pg);
}
pub fn parse_files<'db>(
    db: &'db CodegenDatabase,
    #[cfg(feature = "serialization")] cache: &'db Cache,
    dir: String,
) -> (FilesToParse, Vec<String>) {
    rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 1024 * 10)
        .build_global()
        .unwrap();
    let (tx, rx) = crossbeam::channel::unbounded();
    let mut errors = Vec::new();
    log_languages();
    #[cfg(feature = "serialization")]
    let cache = Cache::new().unwrap();
    #[cfg(feature = "serialization")]
    let cached = get_cached_count(&cache, &files_to_parse);
    let files_to_parse = collect_files(db, dir);
    log::info!("Parsing {} files", files_to_parse.files(db).len());
    parse_files_par(
        db,
        #[cfg(feature = "serialization")]
        &cache,
        files_to_parse,
    );
    drop(tx);
    #[cfg(feature = "serialization")]
    report_cached_count(cached, &files_to_parse.files(db));
    for e in rx.iter() {
        errors.push(e);
    }
    (files_to_parse, errors)
}
