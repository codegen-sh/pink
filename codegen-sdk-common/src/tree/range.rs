use rkyv::{Archive, Deserialize, Serialize};

use crate::Point;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Archive, Deserialize, Serialize)]
pub struct Range {
    start: Point,
    end: Point,
}
impl From<tree_sitter::Range> for Range {
    fn from(value: tree_sitter::Range) -> Self {
        Self {
            start: value.start_point.into(),
            end: value.end_point.into(),
        }
    }
}
impl Range {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}
