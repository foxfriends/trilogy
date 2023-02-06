use super::*;
use crate::Parser;

#[derive(Clone, Debug)]
pub struct Expression {
    pub expression: ValueExpression,
    pub handlers: Vec<Handler>,
}

impl Expression {
    pub(crate) fn parse(_parser: &mut Parser) -> SyntaxResult<Self> {
        todo!()
    }
}
