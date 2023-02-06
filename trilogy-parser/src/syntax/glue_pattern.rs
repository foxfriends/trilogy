use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct GluePattern {
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}
