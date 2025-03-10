use std::path::PathBuf;
#[salsa::input]
pub struct File {
    #[id]
    pub path: PathBuf,
    #[return_ref]
    pub content: String,
    #[id]
    pub root: PathBuf,
}
