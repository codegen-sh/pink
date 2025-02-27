use std::{
    any::Any,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use dashmap::{DashMap, mapref::entry::Entry};

use crate::{File, Input};
#[salsa::db]
trait Db: salsa::Database {
    fn input(&self, path: Input) -> anyhow::Result<File>;
}
#[salsa::db]
#[derive(Default, Clone)]
// Basic Database implementation for Query generation. This is not used for anything else.
pub struct CSTDatabase {
    storage: salsa::Storage<Self>,
    files: DashMap<PathBuf, File>,
    file_watcher: Arc<Mutex<Debouncer<RecommendedWatcher>>>,
}
#[salsa::db]
impl salsa::Database for CSTDatabase {
    fn salsa_event(&self, event: &dyn Fn() -> salsa::Event) {}
}
#[salsa::db]
impl Db for CSTDatabase {
    fn input(&self, path: PathBuf) -> anyhow::Result<File> {
        let path = path
            .canonicalize()
            .map_err(|_| format!("Failed to read {}", path.display()))?;
        Ok(match self.files.entry(path.clone()) {
            // If the file already exists in our cache then just return it.
            Entry::Occupied(entry) => *entry.get(),
            // If we haven't read this file yet set up the watch, read the
            // contents, store it in the cache, and return it.
            Entry::Vacant(entry) => {
                // Set up the watch before reading the contents to try to avoid
                // race conditions.
                let watcher = &mut *self.file_watcher.lock().unwrap();
                watcher
                    .watcher()
                    .watch(&path, RecursiveMode::NonRecursive)
                    .unwrap();
                let contents = std::fs::read_to_string(&path)
                    .map_err(|_| format!("Failed to read {}", path.display()))?;
                *entry.insert(File::new(self, path, contents))
            }
        })
    }
}
