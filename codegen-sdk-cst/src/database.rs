use std::{any::Any, path::PathBuf, sync::Arc};

use dashmap::DashMap;

use crate::File;
#[salsa::db]
#[derive(Default, Clone)]
// Basic Database implementation for Query generation and testing. This is not used for anything else.
pub struct CSTDatabase {
    storage: salsa::Storage<Self>,
    pub files: DashMap<PathBuf, File>,
}
#[salsa::db]
impl salsa::Database for CSTDatabase {
    fn salsa_event(&self, event: &dyn Fn() -> salsa::Event) {
        log::debug!(target: "salsa", "Salsa event: {:?}", event());
    }
}
