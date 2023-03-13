use super::*;

#[derive(Clone, Debug)]
pub struct BinaryDirection {
    pub lhs: Direction,
    pub rhs: Direction,
}

impl BinaryDirection {
    pub fn new(lhs: Direction, rhs: Direction) -> Self {
        Self { lhs, rhs }
    }
}
