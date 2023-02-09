use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct GluePattern {
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}
