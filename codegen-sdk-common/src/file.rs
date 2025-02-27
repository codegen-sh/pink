use std::path::PathBuf;

pub trait File {
    fn path(&self) -> &PathBuf;
    fn content(&self) -> String {
        std::fs::read_to_string(self.path()).unwrap()
    }
}
