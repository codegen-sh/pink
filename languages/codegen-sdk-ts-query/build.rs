use codegen_sdk_common::language::ts_query::Query;
use codegen_sdk_cst_generator::{Config, generate_cst_to_file};
fn main() {
    let config = Config {
        serialize: cfg!(feature = "serialization"),
    };
    env_logger::init();
    generate_cst_to_file(&Query, config.clone()).unwrap_or_else(|e| {
        log::error!("Error generating CST for {}: {}", Query.name(), e);
        panic!("Error generating CST for {}: {}", Query.name(), e);
    });
}
