use super::*;

#[derive(Clone, Debug)]
pub struct GluePattern {
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}
