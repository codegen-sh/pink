use codegen_sdk_common::FileNodeId;
use codegen_sdk_cst::CSTLanguage;
use codegen_sdk_macros::{languages_ast, parse_language};
languages_ast!();

#[salsa::tracked(return_ref)]
pub fn parse_file<'db>(
    db: &'db dyn codegen_sdk_resolution::Db,
    file: codegen_sdk_common::FileNodeId,
) -> Option<ParsedFile<'db>> {
    parse_language!();
    None
}
