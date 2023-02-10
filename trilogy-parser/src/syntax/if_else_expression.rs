use super::{value_expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct IfElseExpression {
    start: Token,
    pub condition: Expression,
    pub when_true: Expression,
    pub when_false: Expression,
}

impl IfElseExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwIf).expect("Caller should have found this");
        let condition = ValueExpression::parse(parser)?;
        parser.expect(KwThen).map_err(|token| {
            parser.expected(token, "expected `then` to follow if expression condition")
        })?;
        let when_true = ValueExpression::parse(parser)?;
        parser.expect(KwElse).map_err(|token| {
            parser.expected(
                token,
                "expected `else`; an if expression always requires an else clause",
            )
        })?;
        let when_false = ValueExpression::parse_precedence(parser, Precedence::Conditional)?;
        Ok(Self {
            start,
            condition: condition.into(),
            when_true: when_true.into(),
            when_false: when_false.into(),
        })
    }
}
