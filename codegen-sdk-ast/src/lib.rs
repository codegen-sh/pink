use codegen_sdk_common::{File, HasNode};
pub use codegen_sdk_cst::*;
pub trait Named {
    fn name(&self) -> &str;
}
impl<T: File> Named for T {
    fn name(&self) -> &str {
        self.path().file_name().unwrap().to_str().unwrap()
    }
}
use std::path::PathBuf;
include!(concat!(env!("OUT_DIR"), "/typescript.rs"));
