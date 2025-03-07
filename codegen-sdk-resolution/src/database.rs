use std::path::PathBuf;

use codegen_sdk_cst::File;
use indicatif::MultiProgress;
#[salsa::db]
pub trait Db: salsa::Database + Send {
    fn input(&self, path: PathBuf) -> anyhow::Result<File>;
    fn get_file(&self, path: PathBuf) -> Option<File>;
    fn multi_progress(&self) -> &MultiProgress;
    fn watch_dir(&mut self, path: PathBuf) -> anyhow::Result<()>;
    fn files(&self) -> indexmap::IndexSet<File>;
}
#[salsa::tracked]
pub fn files(db: &dyn Db) -> indexmap::IndexSet<codegen_sdk_cst::File> {
    db.files()
}
