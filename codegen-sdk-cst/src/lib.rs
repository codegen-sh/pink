#![recursion_limit = "512"]
#![feature(trivial_bounds, extend_one, error_generic_member_access)]
#![allow(unused)]
mod errors;
use std::{any::Any, path::PathBuf};

pub use errors::*;
mod input;
use dashmap::{DashMap, mapref::entry::Entry};
mod database;
use codegen_sdk_common::{ParseError, traits::CSTNode};
pub use database::CSTDatabase;
pub use input::File;
mod language;
pub use codegen_sdk_common::language::LANGUAGES;
pub use language::CSTLanguage;
