use super::{value_expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ReturnExpression {
    start: Token,
    pub expression: Expression,
}

impl ReturnExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwReturn)
            .expect("Caller should have found this");
        let expression = ValueExpression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            start,
            expression: expression.into(),
        })
    }
}
