use std::path::PathBuf;

use codegen_sdk_ast::*;
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use glob::glob;

use crate::database::{CodegenDatabase, Db};
#[salsa::input]
pub struct FilesToParse {
    pub files: Vec<codegen_sdk_ast::input::File>,
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
        .map(|file| db.input(file).unwrap())
        .collect();
    FilesToParse::new(db, files)
}
