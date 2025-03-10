use std::path::PathBuf;

use codegen_sdk_analyzer::ParsedFile;
use codegen_sdk_resolution::CodebaseContext;
use file::File;
use pyo3::{prelude::*, sync::GILProtected};
mod file;
#[pyclass]
struct Codebase {
    codebase: GILProtected<codegen_sdk_analyzer::Codebase>,
}
#[pymethods]
impl Codebase {
    #[new]
    fn new(repo_path: PathBuf) -> Self {
        Self {
            codebase: GILProtected::new(codegen_sdk_analyzer::Codebase::new(repo_path)),
        }
    }
    fn get_file(&self, py: Python<'_>, path: PathBuf) -> PyResult<Option<File>> {
        let file = self.codebase.get(py).get_file(path);
        let file = file.map(|file| File::new(file));
        Ok(file)
    }
}

#[pymodule]
#[pyo3(name = "codegen_sdk_pink")]
fn codegen_sdk_pink(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<Codebase>()?;
    Ok(())
}
