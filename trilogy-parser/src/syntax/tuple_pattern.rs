use super::*;

#[derive(Clone, Debug)]
pub struct TuplePattern {
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}
