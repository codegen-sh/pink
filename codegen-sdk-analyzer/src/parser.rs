use std::path::PathBuf;

use codegen_sdk_common::FileNodeId;
use codegen_sdk_cst::CSTLanguage;
use codegen_sdk_macros::{languages_ast, parse_language};
languages_ast!();

#[salsa::tracked]
pub struct Parsed<'db> {
    #[id]
    path: FileNodeId<'db>,
    #[return_ref]
    #[tracked]
    pub file: Option<ParsedFile<'db>>,
}
#[salsa::tracked(return_ref)]
pub fn parse_file(db: &dyn salsa::Database, file: codegen_sdk_ast::input::File) -> Parsed<'_> {
    parse_language!();
    Parsed::new(db, FileNodeId::new(db, file.path(db)), None)
}
