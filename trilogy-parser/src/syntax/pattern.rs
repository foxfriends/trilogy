use super::*;
use crate::Parser;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Pattern {
    pub disjunction: PatternDisjunction,
}

impl Pattern {
    pub(crate) fn parse(_parser: &mut Parser) -> SyntaxResult<Self> {
        todo!()
    }
}
