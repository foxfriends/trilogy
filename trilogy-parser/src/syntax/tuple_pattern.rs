use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct TuplePattern {
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}
