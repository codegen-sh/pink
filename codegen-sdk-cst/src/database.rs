use std::{any::Any, path::PathBuf, sync::Arc};

use dashmap::{DashMap, mapref::entry::Entry};

use crate::File;
#[salsa::db]
#[derive(Default, Clone)]
// Basic Database implementation for Query generation and testing. This is not used for anything else.
pub struct CSTDatabase {
    storage: salsa::Storage<Self>,
}
#[salsa::db]
impl salsa::Database for CSTDatabase {
    fn salsa_event(&self, event: &dyn Fn() -> salsa::Event) {}
}
