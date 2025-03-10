use std::{path::PathBuf, sync::Arc};

use codegen_sdk_common::language::LANGUAGES;
use codegen_sdk_resolution::CodebaseContext;
use file::File;
use pyo3::{prelude::*, sync::GILProtected};
// use pyo3_stub_gen::{
//     define_stub_info_gatherer,
//     derive::{gen_stub_pyclass, gen_stub_pyclass_enum, gen_stub_pymethods},
// };
mod file;
pub mod python {
    include!(concat!(env!("OUT_DIR"), "/python-bindings.rs"));
}
// #[gen_stub_pyclass_enum]
#[derive(IntoPyObject)]
enum FileEnum {
    Python(crate::python::PythonFile),
    Unknown(crate::file::File),
}
// #[gen_stub_pyclass]
#[pyclass]
struct Codebase {
    codebase: Arc<GILProtected<codegen_sdk_analyzer::Codebase>>,
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
        Ok(self.codebase.get(py).get_file(path).is_some())
    }
    fn get_file(&self, py: Python<'_>, path: PathBuf) -> PyResult<Option<FileEnum>> {
        let path = path.canonicalize()?;
        if self.has_file(py, path.clone())? {
            for language in LANGUAGES.iter() {
                if language
                    .should_parse(&path)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?
                {
                    let file = python::PythonFile::new(path, self.codebase.clone());
                    return Ok(Some(FileEnum::Python(file)));
                }
            }
            let file = crate::file::File::new(path, self.codebase.clone());
            Ok(Some(FileEnum::Unknown(file)))
        } else {
            Ok(None)
        }
    }
}

#[pymodule]
#[pyo3(name = "codegen_sdk_pink")]
fn codegen_sdk_pink(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _ = pyo3_log::try_init();
    m.add_class::<File>()?;
    python::register_python(m)?;
    m.add_class::<Codebase>()?;
    Ok(())
}
// define_stub_info_gatherer!(stub_info);
