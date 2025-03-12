use std::{path::PathBuf, sync::Arc};

use codegen_sdk_resolution::{CodebaseContext, File as _};
use file::File;
use pyo3::{prelude::*, sync::GILProtected};
// use pyo3_stub_gen::{
//     define_stub_info_gatherer,
//     derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods},
// };
mod file;
// #[gen_stub_pyclass_enum]
include!(concat!(env!("OUT_DIR"), "/common-bindings.rs"));

// #[gen_stub_pyclass]
#[pyclass]
struct Codebase {
    codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
}
impl Codebase {
    fn convert_file(&self, path: &PathBuf) -> PyResult<FileEnum> {
        FileEnum::parse(path, self.codebase.clone())
    }
}
// #[gen_stub_pymethods]
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
        Ok(self.codebase.get(py).get_file(&path).is_some())
    }
    fn get_file(&self, py: Python<'_>, path: PathBuf) -> PyResult<Option<FileEnum>> {
        let path = path.canonicalize()?;
        if self.has_file(py, path.clone())? {
            Ok(Some(self.convert_file(&path)?))
        } else {
            Ok(None)
        }
    }
    #[getter]
    fn files(&self, py: Python<'_>) -> PyResult<Vec<FileEnum>> {
        let files = self.codebase.get(py).files();
        Ok(files
            .iter()
            .filter_map(|file| {
                self.convert_file(file.path(self.codebase.get(py).db()))
                    .ok()
            })
            .collect())
    }
}

#[pymodule]
#[pyo3(name = "codegen_sdk_pink")]
fn codegen_sdk_pink(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = pyo3_log::try_init();
    m.add_class::<File>()?;
    register_all(py, m)?;
    m.add_class::<Codebase>()?;
    Ok(())
}
// define_stub_info_gatherer!(stub_info);
