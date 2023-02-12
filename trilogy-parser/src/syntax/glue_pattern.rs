use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct GluePattern {
    pub lhs: Pattern,
    pub rhs: Pattern,
}
