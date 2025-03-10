use std::{hash::Hash, path::PathBuf, sync::Arc};

use bytes::Bytes;
use codegen_sdk_common::{
    ParseError, Tree,
    language::Language,
    traits::{CSTNode, FromNode},
    tree::TreeNode,
};

pub trait CSTLanguage {
    type Types<'db>: TreeNode;
    type Program<'db1>: CSTNode<'db1> + FromNode<'db1, Self::Types<'db1>> + Send;
    fn language() -> &'static Language;
    fn parse<'db>(
        db: &'db dyn salsa::Database,
        content: String,
    ) -> Option<(
        &'db Self::Program<'db>,
        &'db Tree<Self::Types<'db>>,
        indextree::NodeId,
    )>;
    fn parse_file_from_cache<'db>(
        db: &'db dyn salsa::Database,
        file_path: &PathBuf,
        #[cfg(feature = "serialization")] cache: &'db codegen_sdk_common::serialize::Cache,
    ) -> Result<
        Option<(
            &'db Self::Program<'db>,
            &'db Tree<Self::Types<'db>>,
            indextree::NodeId,
        )>,
        ParseError,
    > {
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
    ) -> Result<
        Option<(
            &'db Self::Program<'db>,
            &'db Tree<Self::Types<'db>>,
            indextree::NodeId,
        )>,
        ParseError,
    > {
        if let Some(parsed) = Self::parse_file_from_cache(
            db,
            file_path,
            #[cfg(feature = "serialization")]
            cache,
        )? {
            return Ok(Some(parsed));
        }
        let content = std::fs::read_to_string(file_path)?;
        if let Some(parsed) = Self::parse(db, content) {
            return Ok(Some(parsed));
        }
        Err(ParseError::SyntaxError)
    }

    fn should_parse(file_path: &PathBuf) -> Result<bool, ParseError> {
        Self::language().should_parse(file_path)
    }
}
