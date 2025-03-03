use std::{path::PathBuf, sync::Arc};

use bytes::Bytes;
use codegen_sdk_common::{
    ParseError,
    language::Language,
    traits::{CSTNode, FromNode},
};

pub trait CSTLanguage {
    type Program<'db1>: CSTNode<'db1> + FromNode<'db1> + Send;
    fn language() -> &'static Language;
    fn parse<'db>(db: &'db dyn salsa::Database, content: String)
    -> &'db Option<Self::Program<'db>>;
    fn parse_file_from_cache<'db>(
        db: &'db dyn salsa::Database,
        file_path: &PathBuf,
        #[cfg(feature = "serialization")] cache: &'db codegen_sdk_common::serialize::Cache,
    ) -> Result<&'db Option<Self::Program<'db>>, ParseError> {
        #[cfg(feature = "serialization")]
        {
            let serialized_path = cache.get_path(file_path);
            if serialized_path.exists() {
                let parsed = cache.read_entry::<Self::Program<'db>>(&serialized_path)?;
                return Ok(Some(parsed));
            }
        }
        Ok(&None)
    }
    fn parse_file<'db>(
        db: &'db dyn salsa::Database,
        file_path: &PathBuf,
        #[cfg(feature = "serialization")] cache: &'db codegen_sdk_common::serialize::Cache,
    ) -> Result<&'db Self::Program<'db>, ParseError> {
        if let Some(parsed) = Self::parse_file_from_cache(
            db,
            file_path,
            #[cfg(feature = "serialization")]
            cache,
        )? {
            return Ok(parsed);
        }
        let content = std::fs::read_to_string(file_path)?;
        if let Some(parsed) = Self::parse(db, content) {
            return Ok(parsed);
        }
        Err(ParseError::SyntaxError)
    }

    fn should_parse(file_path: &PathBuf) -> Result<bool, ParseError> {
        Ok(Self::language().file_extensions.contains(
            &file_path
                .extension()
                .ok_or(ParseError::Miscelaneous)?
                .to_str()
                .ok_or(ParseError::Miscelaneous)?,
        ))
    }
}
