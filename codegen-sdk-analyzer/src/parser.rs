use codegen_sdk_cst::CSTLanguage;
#[derive(Debug, Clone, Eq, PartialEq, Hash, salsa::Update)]
pub enum ParsedFile<'db> {
    #[cfg(feature = "typescript")]
    Typescript(codegen_sdk_typescript::ast::TypescriptFile<'db>),
}
#[salsa::tracked]
pub struct Parsed<'db> {
    #[return_ref]
    file: Option<ParsedFile<'db>>,
}
#[salsa::tracked]
pub fn parse_file(db: &dyn salsa::Database, file: codegen_sdk_ast::input::File) -> Parsed<'_> {
    #[cfg(feature = "typescript")]
    if codegen_sdk_typescript::cst::Typescript::should_parse(&file.path(db)).unwrap_or(false) {
        return Parsed::new(
            db,
            Some(ParsedFile::Typescript(codegen_sdk_typescript::ast::parse(
                db, file,
            ))),
        );
    }
    Parsed::new(db, None)
}
