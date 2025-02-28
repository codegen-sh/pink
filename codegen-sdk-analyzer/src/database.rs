use std::{
    path::PathBuf,
    sync::{Arc, Mutex, mpsc::Sender},
    time::Duration,
};

use anyhow::Context;
use codegen_sdk_ast::input::File;
use codegen_sdk_cst::Input;
use dashmap::{DashMap, mapref::entry::Entry};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use notify_debouncer_mini::{
    DebounceEventResult, Debouncer, new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
};
#[salsa::db]
pub trait Db: salsa::Database + Send {
    fn input(&self, path: PathBuf) -> anyhow::Result<File>;
    fn multi_progress(&self) -> &MultiProgress;
}
#[salsa::db]
#[derive(Clone)]
// Basic Database implementation for Query generation. This is not used for anything else.
pub struct CodegenDatabase {
    storage: salsa::Storage<Self>,
    files: DashMap<PathBuf, File>,
    multi_progress: MultiProgress,
    file_watcher: Arc<Mutex<Debouncer<RecommendedWatcher>>>,
}
impl CodegenDatabase {
    pub fn new(tx: Sender<DebounceEventResult>) -> Self {
        let logger =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .build();
        let level = logger.filter();
        let multi_progress = MultiProgress::new();
        log::set_max_level(level);
        LogWrapper::new(multi_progress.clone(), logger);
        Self {
            file_watcher: Arc::new(Mutex::new(
                new_debouncer(Duration::from_secs(1), tx).unwrap(),
            )),
            storage: salsa::Storage::default(),
            multi_progress,
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
            log::debug!("{:?}", event);
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
    fn multi_progress(&self) -> &MultiProgress {
        &self.multi_progress
    }
}
