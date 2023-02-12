use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TuplePattern {
    pub lhs: Pattern,
    pub rhs: Pattern,
}
