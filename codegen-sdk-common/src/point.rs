use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Archive, Deserialize, Serialize)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}
impl From<tree_sitter::Point> for Point {
    fn from(value: tree_sitter::Point) -> Self {
        Point {
            row: value.row,
            column: value.column,
        }
    }
}
