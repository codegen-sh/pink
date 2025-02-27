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
    fn parse<'db>(
        db: &'db dyn salsa::Database,
        content: &str,
    ) -> Result<Self::Program<'db>, ParseError> {
        let buffer = Bytes::from(content.as_bytes().to_vec());
        let tree = Self::language().parse_tree_sitter(content)?;
        if tree.root_node().has_error() {
            Err(ParseError::SyntaxError)
        } else {
            let buffer = Arc::new(buffer);
            Self::Program::from_node(db, tree.root_node(), &buffer)
        }
    }
    fn parse_file_from_cache<'db>(
        db: &'db dyn salsa::Database,
        file_path: &PathBuf,
        #[cfg(feature = "serialization")] cache: &'db codegen_sdk_common::serialize::Cache,
    ) -> Result<Option<Self::Program<'db>>, ParseError> {
        #[cfg(feature = "serialization")]
        {
            let serialized_path = cache.get_path(file_path);
            if serialized_path.exists() {
                let parsed = cache.read_entry::<Self::Program<'db>>(&serialized_path)?;
                return Ok(Some(parsed));
            }
        }
        Ok(None)
    }
    fn parse_file<'db>(
        db: &'db dyn salsa::Database,
        file_path: &PathBuf,
        #[cfg(feature = "serialization")] cache: &'db codegen_sdk_common::serialize::Cache,
    ) -> Result<Self::Program<'db>, ParseError> {
        if let Some(parsed) = Self::parse_file_from_cache(
            db,
            file_path,
            #[cfg(feature = "serialization")]
            cache,
        )? {
            return Ok(parsed);
        }
        let content = std::fs::read_to_string(file_path)?;
        let parsed = Self::parse(db, &content)?;
        Ok(parsed)
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
