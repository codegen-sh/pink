use std::path::PathBuf;

use codegen_sdk_ast::input::File;
use indicatif::MultiProgress;
#[salsa::db]
pub trait Db: salsa::Database + Send {
    fn input(&self, path: PathBuf) -> anyhow::Result<File>;
    fn multi_progress(&self) -> &MultiProgress;
    fn watch_dir(&mut self, path: PathBuf) -> anyhow::Result<()>;
    fn files(&self) -> Vec<File>;
}
