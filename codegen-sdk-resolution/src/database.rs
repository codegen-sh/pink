use std::path::PathBuf;

use codegen_sdk_cst::File;
use indicatif::MultiProgress;
#[salsa::db]
pub trait Db: salsa::Database + Send {
    fn input(&self, path: &PathBuf) -> anyhow::Result<File>;
    fn get_file(&self, path: &PathBuf) -> Option<File>;
    fn multi_progress(&self) -> &MultiProgress;
    fn watch_dir(&mut self, path: PathBuf) -> anyhow::Result<()>;
    fn files(&self) -> codegen_sdk_common::hash::FxHashSet<codegen_sdk_common::FileNodeId>;
    fn get_file_for_id(&self, id: codegen_sdk_common::FileNodeId) -> Option<File> {
        self.get_file(&id.path(self))
    }
}
#[salsa::tracked]
pub fn files<'db>(
    db: &'db dyn Db,
) -> codegen_sdk_common::hash::FxHashSet<codegen_sdk_common::FileNodeId> {
    db.files()
}
#[salsa::db]
impl Db for codegen_sdk_cst::CSTDatabase {
    fn input(&self, path: &PathBuf) -> anyhow::Result<File> {
        let content = std::fs::read_to_string(path)?;
        let file =
            codegen_sdk_cst::File::new(self, path.canonicalize().unwrap(), content, PathBuf::new());
        Ok(file)
    }
    fn multi_progress(&self) -> &MultiProgress {
        unimplemented!()
    }
    fn watch_dir(&mut self, _path: PathBuf) -> anyhow::Result<()> {
        unimplemented!()
    }
    fn files(&self) -> codegen_sdk_common::hash::FxHashSet<codegen_sdk_common::FileNodeId> {
        let path = PathBuf::from(".");
        let files = std::fs::read_dir(path).unwrap();
        let mut set = codegen_sdk_common::hash::FxHashSet::default();
        for file in files {
            let file = file.unwrap();
            let path = file.path().canonicalize().unwrap();
            if path.is_file() {
                set.insert(codegen_sdk_common::FileNodeId::new(self, path));
            }
        }
        set
    }
    fn get_file(&self, path: &PathBuf) -> Option<File> {
        self.input(path).ok()
    }
}
