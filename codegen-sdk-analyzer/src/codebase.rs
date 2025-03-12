use std::path::PathBuf;

use anyhow::Context;
#[cfg(feature = "serialization")]
use codegen_sdk_common::serialization::Cache;
use codegen_sdk_resolution::{CodebaseContext, Db};
use discovery::FilesToParse;
use notify_debouncer_mini::DebounceEventResult;
use salsa::{Database, Setter};

use crate::{ParsedFile, database::CodegenDatabase, parser::parse_file};
mod discovery;
mod parser;
use parser::execute_op_with_progress;

pub struct Codebase {
    db: CodegenDatabase,
    root: PathBuf,
    rx: crossbeam_channel::Receiver<DebounceEventResult>,
    #[cfg(feature = "serialization")]
    cache: Cache,
}

impl Codebase {
    pub fn new(root: PathBuf) -> Self {
        let root = root.canonicalize().unwrap();
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut db = CodegenDatabase::new(tx, root.clone());
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
                    file.set_content(&mut self.db).to(contents);
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

    fn discover(&self) -> FilesToParse {
        discovery::collect_files(&self.db, &self.root)
    }

    pub fn errors(&self) -> Vec<()> {
        let mut errors = Vec::new();
        for file in self.db.files() {
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
    fn _db(&self) -> &dyn Db {
        &self.db
    }
    pub fn execute_op_with_progress<T: Send + Sync>(
        &self,
        name: &str,
        parallel: bool,
        op: fn(&dyn Db, codegen_sdk_common::FileNodeId) -> T,
    ) -> Vec<T> {
        execute_op_with_progress(
            self._db(),
            codegen_sdk_resolution::files(self._db()),
            name,
            parallel,
            op,
        )
    }
}
impl CodebaseContext for Codebase {
    type File<'a> = ParsedFile<'a>;
    type Db = CodegenDatabase;
    fn root_path(&self) -> PathBuf {
        self.root.clone()
    }
    fn files<'a>(&'a self) -> Vec<&'a Self::File<'a>> {
        let mut files = Vec::new();
        for file in self.discover().files(&self.db) {
            if let Some(file) = self.get_file(&file.path(&self.db)) {
                files.push(file);
            }
        }
        files
    }
    fn db(&self) -> &dyn Db {
        &self.db
    }
    fn get_file<'a>(&'a self, path: &PathBuf) -> Option<&'a Self::File<'a>> {
        if let Ok(path) = path.canonicalize() {
            let file = self.db.files.get(&path);
            if let Some(_) = file {
                let file_id = codegen_sdk_common::FileNodeId::new(&self.db, path);
                return parse_file(&self.db, file_id).file(&self.db).as_ref();
            }
        }
        None
    }
    fn attach<T>(&self, op: impl FnOnce(&Self::Db) -> T) -> T {
        self.db.attach(op)
    }
}
