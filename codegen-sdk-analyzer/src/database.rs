use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Context;
use codegen_sdk_ast::input::File;
use codegen_sdk_cst::Input;
use dashmap::{DashMap, mapref::entry::Entry};
use indicatif::MultiProgress;
use notify_debouncer_mini::{
    Config, DebounceEventResult, Debouncer, new_debouncer_opt,
    notify::{RecommendedWatcher, RecursiveMode},
};

use crate::progress::get_multi_progress;
#[salsa::db]
pub trait Db: salsa::Database + Send {
    fn input(&self, path: PathBuf) -> anyhow::Result<File>;
    fn multi_progress(&self) -> &MultiProgress;
    fn watch_dir(&mut self, path: PathBuf) -> anyhow::Result<()>;
}
#[salsa::db]
#[derive(Clone)]
// Basic Database implementation for Query generation. This is not used for anything else.
pub struct CodegenDatabase {
    storage: salsa::Storage<Self>,
    pub files: DashMap<PathBuf, File>,
    dirs: Vec<PathBuf>,
    multi_progress: MultiProgress,
    file_watcher: Arc<Mutex<Debouncer<RecommendedWatcher>>>,
}
fn get_watcher(
    tx: crossbeam_channel::Sender<DebounceEventResult>,
) -> Arc<Mutex<Debouncer<RecommendedWatcher>>> {
    let config = Config::default()
        .with_batch_mode(true)
        .with_timeout(Duration::from_secs(2));
    Arc::new(Mutex::new(new_debouncer_opt(config, tx).unwrap()))
}
impl CodegenDatabase {
    pub fn new(tx: crossbeam_channel::Sender<DebounceEventResult>) -> Self {
        let multi_progress = get_multi_progress();
        Self {
            file_watcher: get_watcher(tx),
            storage: salsa::Storage::default(),
            multi_progress,
            files: DashMap::new(),
            dirs: Vec::new(),
        }
    }
    fn _watch_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        for dir in self.dirs.iter() {
            if path.starts_with(dir) {
                return Ok(());
            }
        }
        let watcher = &mut *self.file_watcher.lock().unwrap();
        watcher
            .watcher()
            .watch(&path, RecursiveMode::NonRecursive)
            .unwrap();
        Ok(())
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
    fn watch_dir(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let path = path.canonicalize()?;
        let watcher = &mut *self.file_watcher.lock().unwrap();
        watcher
            .watcher()
            .watch(&path, RecursiveMode::Recursive)
            .unwrap();
        self.dirs.push(path);
        Ok(())
    }
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
                self._watch_file(&path)?;
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
