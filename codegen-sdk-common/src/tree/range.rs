use rkyv::{Archive, Deserialize, Serialize};

use crate::Point;
#[salsa::interned]
#[derive(Archive, Deserialize, Serialize)]
pub struct Range<'db> {
    start: Point<'db>,
    end: Point<'db>,
}
impl<'db> Range<'db> {
    pub fn from_points(db: &'db dyn salsa::Database, start: Point<'db>, end: Point<'db>) -> Self {
        Self::new(db, start, end)
    }
    pub fn from_tree_sitter(db: &'db dyn salsa::Database, value: tree_sitter::Range) -> Self {
        Self::from_points(
            db,
            Point::from(db, value.start_point),
            Point::from(db, value.end_point),
        )
    }
}
