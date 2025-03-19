use codegen_sdk_ast::{Definitions, References};
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use codegen_sdk_cst::LANGUAGES;
use codegen_sdk_resolution::Db;
use indicatif::{ProgressBar, ProgressStyle};

use super::discovery::{FilesToParse, log_languages};
use crate::{ParsedFile, parser::parse_file};
pub fn execute_op_with_progress<
    Database: Db + ?Sized + 'static,
    Input: Send + Sync,
    T: Send + Sync,
>(
    db: &Database,
    files: codegen_sdk_common::hash::FxHashSet<Input>,
    name: &str,
    parallel: bool,
    op: fn(&Database, Input) -> T,
) -> Vec<T> {
    let multi = db.multi_progress();
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {wide_bar} {msg} [{per_sec}] [estimated time remaining: {eta}]",
    )
    .unwrap();
    let pg = multi.add(
        ProgressBar::new(files.len() as u64)
            .with_style(style)
            .with_message(name.to_string()),
    );
    let inputs = files
        .into_iter()
        .map(|file| (&pg, file, op))
        .collect::<Vec<_>>();
    let results: Vec<_> = if parallel {
        salsa::par_map(db, inputs, move |db, input| {
            let (pg, file, op) = input;
            let res = op(
                db,
                #[cfg(feature = "serialization")]
                &cache,
                file,
            );
            pg.inc(1);
            res
        })
    } else {
        inputs
            .into_iter()
            .map(|input| {
                let (pg, file, op) = input;
                let res = op(
                    db,
                    #[cfg(feature = "serialization")]
                    &cache,
                    file,
                );
                pg.inc(1);
                res
            })
            .collect()
    };
    pg.finish();
    results
}
// #[salsa::tracked]
// fn parse_files_par(db: &dyn Db, files: FilesToParse) {
//     let _: Vec<_> = execute_op_with_progress(db, files, "Parsing Files", |db, file| {
//         parse_file(db, file);
//     });
// }
#[salsa::tracked]
fn parse_files_definitions_par(db: &dyn Db, files: FilesToParse) {
    let ids = files
        .files(db)
        .iter()
        .map(|input| codegen_sdk_common::FileNodeId::new(db, input.path(db)))
        .collect::<codegen_sdk_common::hash::FxHashSet<_>>();
    let _: Vec<_> = execute_op_with_progress(db, ids, "Parsing Files", true, |db, input| {
        let file = parse_file(db, input.clone());
        if let Some(parsed) = file {
            #[cfg(feature = "typescript")]
            if let ParsedFile::Typescript(parsed) = parsed {
                parsed.definitions(db);
                parsed.references(db);
            }
            #[cfg(feature = "python")]
            if let ParsedFile::Python(parsed) = parsed {
                parsed.definitions(db);
                parsed.references(db);
                // let deps = codegen_sdk_python::ast::dependencies(db, input);
                // for dep in deps.dependencies(db).keys() {
                //     codegen_sdk_resolution::ast::references_impl(db, dep);
                // }
            }
        }
        ()
    });
}
#[salsa::tracked]
fn compute_dependencies_par(db: &dyn Db) {
    log::info!("Computing Dependencies");
    #[cfg(feature = "python")]
    let _ = codegen_sdk_python::ast::dependency_matrix(db);
    log::info!("Done Computing Dependencies");
}

pub fn parse_files<'db>(
    db: &'db dyn Db,
    #[cfg(feature = "serialization")] cache: &'db Cache,
    files_to_parse: FilesToParse,
) -> () {
    let _ = rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 1024 * 10)
        .build_global();
    log_languages();
    #[cfg(feature = "serialization")]
    let cache = Cache::new().unwrap();
    #[cfg(feature = "serialization")]
    let cached = get_cached_count(&cache, &files_to_parse);
    log::info!("Parsing {} files", files_to_parse.files(db).len());
    for language in LANGUAGES.iter() {
        let mut count = 0;
        for file in files_to_parse.files(db).iter() {
            if language.should_parse(&file.path(db)).unwrap() {
                count += 1;
            }
        }
        log::info!("{} files to parse for {}", count, language.name());
    }
    parse_files_definitions_par(
        db,
        #[cfg(feature = "serialization")]
        &cache,
        files_to_parse,
    );
    compute_dependencies_par(
        db,
        #[cfg(feature = "serialization")]
        &cache,
    );
    #[cfg(feature = "serialization")]
    report_cached_count(cached, &files_to_parse.files(db));
}
