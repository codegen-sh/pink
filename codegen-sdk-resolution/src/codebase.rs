use std::path::PathBuf;

use codegen_sdk_common::FileNodeId;

use crate::Db;
// Not sure what to name this
// Equivalent to CodebaseGraph/CodebaseContext in the SDK
pub trait CodebaseContext {
    type File<'a>
    where
        Self: 'a;
    fn files<'a>(&'a self) -> Vec<&'a Self::File<'a>>;
    fn db(&self) -> &dyn Db;
    fn get_file<'a>(&'a self, path: PathBuf) -> Option<&'a Self::File<'a>>;
    fn get_file_for_id<'a>(&'a self, id: FileNodeId) -> Option<&'a Self::File<'a>> {
        self.get_file(id.path(self.db()))
    }
    fn get_raw_file_for_id<'a>(&'a self, id: FileNodeId) -> Option<codegen_sdk_ast::input::File> {
        self.get_raw_file(id.path(self.db()))
    }
    fn get_raw_file<'a>(&'a self, path: PathBuf) -> Option<codegen_sdk_ast::input::File> {
        self.db().input(path).ok()
    }
    fn root_path(&self) -> PathBuf;
}
