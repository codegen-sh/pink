use std::path::PathBuf;
#[salsa::input]
pub struct Input {
    pub content: String,
    pub root: PathBuf,
}
