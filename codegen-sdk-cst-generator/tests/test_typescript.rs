use codegen_sdk_common::language::typescript::Typescript;
use codegen_sdk_cst_generator::{Config, generate_cst};

#[test_log::test]
fn test_generate_cst() {
    let language = &Typescript;
    let cst = generate_cst(&language, Config::default()).unwrap();
    log::info!("{}", cst);
}
