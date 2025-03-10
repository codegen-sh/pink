use std::path::PathBuf;

use codegen_sdk_analyzer::ParsedFile;
use pyo3::pyclass;
#[pyclass]
pub struct File {
    path: PathBuf,
}
impl File {
    pub fn new(file: &ParsedFile) -> Self {
        Self {
            path: file.path.clone(),
        }
    }
}
