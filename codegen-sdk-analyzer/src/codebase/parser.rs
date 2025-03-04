use codegen_sdk_ast::{Definitions, References, input::File};
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use indicatif::{ProgressBar, ProgressStyle};

use super::discovery::{FilesToParse, log_languages};
use crate::{
    ParsedFile,
    database::{CodegenDatabase, Db},
    parser::parse_file,
};
fn execute_op_with_progress<Database: Db + ?Sized + 'static, T: Send + Sync>(
    db: &Database,
    files: FilesToParse,
    name: &str,
    op: fn(&Database, File) -> T,
) -> Vec<T> {
    let multi = db.multi_progress();
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {wide_bar} {msg} [{per_sec}] [estimated time remaining: {eta}]",
    )
    .unwrap();
    let pg = multi.add(
        ProgressBar::new(files.files(db).len() as u64)
            .with_style(style)
            .with_message(name.to_string()),
    );
    let inputs = files
        .files(db)
        .into_iter()
        .map(|file| (&pg, file, op))
        .collect::<Vec<_>>();
    let results: Vec<T> = salsa::par_map(db, inputs, move |db, input| {
        let (pg, file, op) = input;
        let res = op(
            db,
            #[cfg(feature = "serialization")]
            &cache,
            file,
        );
        pg.inc(1);
        res
    });
    pg.finish();
    multi.remove(&pg);
    results
}
#[salsa::tracked]
fn parse_files_par(db: &dyn Db, files: FilesToParse) {
    let _: Vec<_> = execute_op_with_progress(db, files, "Parsing Files", |db, file| {
        parse_file(db, file);
    });
}
#[salsa::tracked]
fn parse_files_definitions_par(db: &dyn Db, files: FilesToParse) {
    let _: Vec<_> = execute_op_with_progress(db, files, "Parsing Definitions", |db, file| {
        let file = parse_file(db, file);
        if let Some(parsed) = file.file(db) {
            #[cfg(feature = "typescript")]
            if let ParsedFile::Typescript(parsed) = parsed {
                parsed.definitions(db);
                parsed.references(db);
            }
            #[cfg(feature = "python")]
            if let ParsedFile::Python(parsed) = parsed {
                parsed.definitions(db);
                parsed.references(db);
            }
        }
        ()
    });
}
pub fn parse_files<'db>(
    db: &'db CodegenDatabase,
    #[cfg(feature = "serialization")] cache: &'db Cache,
    files_to_parse: FilesToParse,
) -> () {
    rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 1024 * 10)
        .build_global()
        .unwrap();
    log_languages();
    #[cfg(feature = "serialization")]
    let cache = Cache::new().unwrap();
    #[cfg(feature = "serialization")]
    let cached = get_cached_count(&cache, &files_to_parse);
    log::info!("Parsing {} files", files_to_parse.files(db).len());
    parse_files_par(
        db,
        #[cfg(feature = "serialization")]
        &cache,
        files_to_parse,
    );
    log::info!("Parsing definitions");
    parse_files_definitions_par(
        db,
        #[cfg(feature = "serialization")]
        &cache,
        files_to_parse,
    );
    #[cfg(feature = "serialization")]
    report_cached_count(cached, &files_to_parse.files(db));
}
