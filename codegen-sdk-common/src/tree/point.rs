use rkyv::{Archive, Deserialize, Serialize};

#[salsa::tracked]
#[derive(Archive, Deserialize, Serialize)]
pub struct Point<'db> {
    #[tracked]
    pub row: usize,
    #[tracked]
    pub column: usize,
}
impl<'db> Point<'db> {
    pub fn from(db: &'db dyn salsa::Database, value: tree_sitter::Point) -> Self {
        Self::new(db, value.row, value.column)
    }
}
