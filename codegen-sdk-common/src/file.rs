use std::path::PathBuf;

use crate::ParseError;

pub trait File {
    fn path(&self) -> &PathBuf;
    fn content(&self) -> String {
        std::fs::read_to_string(self.path()).unwrap()
    }
    fn parse(path: &PathBuf) -> Result<Self, ParseError>
    where
        Self: Sized;
}
