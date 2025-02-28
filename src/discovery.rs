use std::path;

use codegen_sdk_analyzer::CodegenDatabase;
use codegen_sdk_ast::*;
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialize::Cache;
use glob::glob;

#[salsa::input]
pub struct FilesToParse {
    pub files: Vec<path::PathBuf>,
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

pub fn collect_files(db: &CodegenDatabase, dir: String) -> FilesToParse {
    let mut files = Vec::new();
    for language in LANGUAGES.iter() {
        for extension in language.file_extensions.iter() {
            files.extend(glob(&format!("{dir}**/*.{}", extension)).unwrap());
        }
    }

    let files = files.into_iter().filter_map(|file| file.ok()).collect();
    FilesToParse::new(db, files)
}
