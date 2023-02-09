use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TuplePattern {
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}
