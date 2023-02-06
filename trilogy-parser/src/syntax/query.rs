use super::*;
use crate::Parser;

#[derive(Clone, Debug, Spanned)]
pub struct Query {
    pub disjunction: Vec<QueryDisjunction>,
}

impl Query {
    pub(crate) fn parse(_parser: &mut Parser) -> SyntaxResult<Self> {
        todo!()
    }
}
