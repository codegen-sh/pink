#![feature(error_generic_member_access)]
mod errors;
pub mod language;
pub mod traits;
pub mod utils;
pub use errors::*;
pub use language::Language;
pub use traits::*;
pub use utils::*;
mod file;
pub use file::File;
pub mod parser;
#[macro_use]
extern crate lazy_static;
pub mod naming;
