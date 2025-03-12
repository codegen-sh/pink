use std::path::PathBuf;

use codegen_sdk_ast::*;
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use codegen_sdk_resolution::Db;
use glob::glob;

use crate::database::CodegenDatabase;
#[salsa::input]
pub struct FilesToParse {
    pub files: codegen_sdk_common::hash::FxHashSet<codegen_sdk_cst::File>,
    pub root: PathBuf,
}
pub fn log_languages() {
    for language in LANGUAGES.iter() {
        log::info!(
            "Supported language: {} with extensions: {:?}",
            language.name(),
            language.file_extensions
        );
    }
}

pub fn collect_files(db: &CodegenDatabase, dir: &PathBuf) -> FilesToParse {
    let mut files = Vec::new();
    let dir = dir.canonicalize().unwrap();
    for language in LANGUAGES.iter() {
        for extension in language.file_extensions.iter() {
            files.extend(
                glob(
                    &dir.join(format!("**/*.{extension}", extension = extension))
                        .to_str()
                        .unwrap(),
                )
                .unwrap(),
            );
        }
    }

    let files = files
        .into_iter()
        .filter_map(|file| file.ok())
        .filter(|file| !file.is_dir() && !file.is_symlink())
        .filter_map(|file| file.canonicalize().ok())
        .map(|file| db.input(&file).unwrap())
        .collect();
    FilesToParse::new(db, files, dir)
}
