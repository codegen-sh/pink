use std::path::PathBuf;

use codegen_sdk_cst::File;
use indicatif::MultiProgress;
#[salsa::db]
pub trait Db: salsa::Database + Send {
    fn input(&self, path: PathBuf) -> anyhow::Result<File>;
    fn get_file(&self, path: PathBuf) -> Option<File>;
    fn multi_progress(&self) -> &MultiProgress;
    fn watch_dir(&mut self, path: PathBuf) -> anyhow::Result<()>;
    fn files(&self) -> codegen_sdk_common::hash::FxHashSet<codegen_sdk_common::FileNodeId<'_>>;
    fn get_file_for_id(&self, id: codegen_sdk_common::FileNodeId<'_>) -> Option<File> {
        self.get_file(id.path(self))
    }
}
#[salsa::tracked]
pub fn files<'db>(
    db: &'db dyn Db,
) -> codegen_sdk_common::hash::FxHashSet<codegen_sdk_common::FileNodeId<'db>> {
    db.files()
}
