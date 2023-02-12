use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PatternDisjunction {
    pub lhs: Pattern,
    pub rhs: Pattern,
}
