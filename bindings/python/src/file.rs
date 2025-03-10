use std::{path::PathBuf, sync::Arc};

use pyo3::{pyclass, sync::GILProtected};
#[pyclass]
pub struct File {
    path: PathBuf,
    codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
}
impl File {
    pub fn new(path: PathBuf, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> Self {
        Self { path, codebase }
    }
}
