use std::path::PathBuf;

use codegen_sdk_cst::Input;
#[salsa::input]
pub struct File {
    #[id]
    pub path: PathBuf,
    // #[return_ref]
    pub contents: Input,
}
