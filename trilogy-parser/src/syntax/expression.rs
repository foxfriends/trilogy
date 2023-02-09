use super::*;
use crate::Parser;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
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

impl From<ValueExpression> for Expression {
    fn from(expression: ValueExpression) -> Self {
        Self {
            expression,
            handlers: vec![],
        }
    }
}
