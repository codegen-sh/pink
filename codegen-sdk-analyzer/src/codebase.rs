use std::path::PathBuf;

use anyhow::Context;
use codegen_sdk_ast::Input;
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialization::Cache;
use discovery::FilesToParse;
use notify_debouncer_mini::DebounceEventResult;
use salsa::Setter;

use crate::{
    ParsedFile,
    database::{CodegenDatabase, Db},
    parser::parse_file,
};
mod discovery;
mod parser;
pub struct Codebase {
    db: CodegenDatabase,
    root: PathBuf,
    rx: crossbeam_channel::Receiver<DebounceEventResult>,
    #[cfg(feature = "serialization")]
    cache: Cache,
}

impl Codebase {
    pub fn new(root: PathBuf) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut db = CodegenDatabase::new(tx);
        db.watch_dir(PathBuf::from(&root)).unwrap();
        let codebase = Self { db, root, rx };
        codebase.sync();
        codebase
    }
    pub fn check_update(&mut self) -> anyhow::Result<()> {
        for event in self.rx.recv()?.unwrap() {
            match event.path.canonicalize() {
                Ok(path) => {
                    log::info!("File changed: {}", path.display());
                    let file = match self.db.files.get(&path) {
                        Some(file) => *file,
                        None => continue,
                    };
                    // `path` has changed, so read it and update the contents to match.
                    // This creates a new revision and causes the incremental algorithm
                    // to kick in, just like any other update to a salsa input.
                    let contents = std::fs::read_to_string(path)
                        .with_context(|| format!("Failed to read file {}", event.path.display()))?;
                    let input = Input::new(&self.db, contents);
                    file.set_contents(&mut self.db).to(input);
                }
                Err(e) => {
                    log::error!(
                        "Failed to canonicalize path {} for file {}",
                        e,
                        event.path.display()
                    );
                }
            }
        }
        Ok(())
    }
    pub fn get_file(&self, path: PathBuf) -> Option<&ParsedFile<'_>> {
        let file = self.db.files.get(&path);
        if let Some(file) = file {
            return parse_file(&self.db, file.clone()).file(&self.db).as_ref();
        }
        None
    }
    fn discover(&self) -> FilesToParse {
        discovery::collect_files(&self.db, &self.root)
    }
    pub fn files(&self) -> Vec<&ParsedFile<'_>> {
        let mut files = Vec::new();
        for file in self.discover().files(&self.db) {
            if let Some(file) = self.get_file(file.path(&self.db)) {
                files.push(file);
            }
        }
        files
    }
    pub fn errors(&self) -> Vec<()> {
        let mut errors = Vec::new();
        for file in self.discover().files(&self.db) {
            if self.get_file(file.path(&self.db)).is_none() {
                errors.push(());
            }
        }
        errors
    }
    pub fn sync(&self) -> () {
        let files = self.discover();
        parser::parse_files(
            &self.db,
            #[cfg(feature = "serialization")]
            &self.cache,
            files,
        )
    }
    pub fn db(&self) -> &CodegenDatabase {
        &self.db
    }
}
