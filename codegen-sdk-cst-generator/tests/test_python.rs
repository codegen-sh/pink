use codegen_sdk_common::language::python::Python;
use codegen_sdk_cst_generator::{Config, generate_cst};

#[test_log::test]
fn test_generate_cst() {
    let language = &Python;
    let cst = generate_cst(&language, Config::default()).unwrap();
    log::info!("{}", cst);
}
