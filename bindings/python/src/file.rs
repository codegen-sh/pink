use std::{path::PathBuf, sync::Arc};

use pyo3::{prelude::*, sync::GILProtected};
#[pyclass]
pub struct File {
    path: PathBuf,
    _codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
}
impl File {
    pub fn new(path: PathBuf, codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>) -> Self {
        Self {
            path,
            _codebase: codebase,
        }
    }
}
#[pymethods]
impl File {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn content(&self) -> PyResult<String> {
        let content = std::fs::read_to_string(&self.path)?;
        Ok(content)
    }
    pub fn content_bytes(&self) -> PyResult<Vec<u8>> {
        let content = std::fs::read(&self.path)?;
        Ok(content)
    }
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }
}
