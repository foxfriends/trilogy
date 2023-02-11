use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct FnExpression {
    start: Token,
    pub parameters: Vec<Pattern>,
    pub body: Expression,
}

impl FnExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwFn).expect("Caller should have found this");
        let mut parameters = vec![];
        loop {
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(OpDot).is_ok() {
                break;
            }
        }
        let body = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            start,
            parameters,
            body,
        })
    }
}
