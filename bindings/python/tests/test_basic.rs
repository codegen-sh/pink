// use pyo3::{ffi::c_str, prelude::*};
// #[test_log::test]

// fn test_basic() {
//     pyo3::prepare_freethreaded_python();
//     Python::with_gil(|py| {
//         let module = PyModule::from_code(
//             py,
//             c_str!(
//                 r#"
// from codegen_sdk_pink import Codebase

// codebase = Codebase("/Users/ellen/workspace/scratch/codegen-sdk/src")
// print(len(codebase.files))
// for file in codebase.files:
//     print(file)
//     print(file.path)
//     print(file.classes)
// "#
//             ),
//             c_str!("test.py"),
//             c_str!("test"),
//         )
//         .unwrap();
//     });
// }
