use super::*;
use crate::Parser;

#[derive(Clone, Debug)]
pub struct Expression {
    pub expression: ValueExpression,
    pub handlers: Vec<Handler>,
}

impl Expression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let expression = ValueExpression::parse(parser)?;
        Ok(Self {
            expression,
            handlers: vec![],
        })
    }
}
