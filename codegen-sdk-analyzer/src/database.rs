use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::Sender},
    time::Duration,
};

use anyhow::Context;
use codegen_sdk_ast::input::File;
use codegen_sdk_cst::Input;
use dashmap::{DashMap, mapref::entry::Entry};
use notify_debouncer_mini::{
    DebounceEventResult, Debouncer, new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
};
#[salsa::db]
pub trait Db: salsa::Database + Send {
    fn input(&self, path: PathBuf) -> anyhow::Result<File>;
}
#[salsa::db]
#[derive(Clone)]
// Basic Database implementation for Query generation. This is not used for anything else.
pub struct CodegenDatabase {
    storage: salsa::Storage<Self>,
    files: DashMap<PathBuf, File>,
    logs: Arc<Mutex<Vec<String>>>,
    file_watcher: Arc<Mutex<Debouncer<RecommendedWatcher>>>,
}
impl CodegenDatabase {
    pub fn new(tx: Sender<DebounceEventResult>) -> Self {
        Self {
            file_watcher: Arc::new(Mutex::new(
                new_debouncer(Duration::from_secs(1), tx).unwrap(),
            )),
            storage: salsa::Storage::default(),
            logs: Default::default(),
            files: DashMap::new(),
        }
    }
}
#[salsa::db]
impl salsa::Database for CodegenDatabase {
    fn salsa_event(&self, event: &dyn Fn() -> salsa::Event) {
        // don't log boring events
        let event = event();
        if let salsa::EventKind::WillExecute { .. } = event.kind {
            self.logs.lock().unwrap().push(format!("{:?}", event));
        }
    }
}
#[salsa::db]
impl Db for CodegenDatabase {
    fn input(&self, path: PathBuf) -> anyhow::Result<File> {
        let path = path
            .canonicalize()
            .with_context(|| format!("Failed to read {}", path.display()))?;
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
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                let input = Input::new(self, contents);
                *entry.insert(File::new(self, path, input))
            }
        })
    }
}
