use std::{path::PathBuf, sync::Arc};

use codegen_sdk_analyzer::ParsedFile;
use codegen_sdk_resolution::CodebaseContext;
use file::File;
use pyo3::{prelude::*, sync::GILProtected};
mod file;
#[pyclass]
struct Codebase {
    codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
}
#[pymethods]
impl Codebase {
    #[new]
    fn new(py: Python<'_>, repo_path: PathBuf) -> Self {
        let codebase = py.allow_threads(|| codegen_sdk_analyzer::Codebase::new(repo_path));
        Self {
            codebase: Arc::new(GILProtected::new(codebase)),
        }
    }
    fn has_file(&self, py: Python<'_>, path: PathBuf) -> PyResult<bool> {
        let path = path.canonicalize()?;
        Ok(self.codebase.get(py).get_file(path).is_some())
    }
    fn get_file(&self, py: Python<'_>, path: PathBuf) -> PyResult<Option<File>> {
        let path = path.canonicalize()?;
        if self.has_file(py, path.clone())? {
            let file = File::new(path, self.codebase.clone());
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }
}

#[pymodule]
#[pyo3(name = "codegen_sdk_pink")]
fn codegen_sdk_pink(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<Codebase>()?;
    Ok(())
}
