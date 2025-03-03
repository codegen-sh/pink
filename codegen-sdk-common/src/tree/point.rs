use rkyv::{Archive, Deserialize, Serialize};

#[salsa::interned]
#[derive(Archive, Deserialize, Serialize)]
pub struct Point<'db> {
    pub row: usize,
    pub column: usize,
}
impl<'db> Point<'db> {
    pub fn from(db: &'db dyn salsa::Database, value: tree_sitter::Point) -> Self {
        Self::new(db, value.row, value.column)
    }
}
