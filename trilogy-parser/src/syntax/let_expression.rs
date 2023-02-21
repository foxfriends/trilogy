use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct LetExpression {
    start: Token,
    pub query: Query,
    pub body: Expression,
}

impl LetExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwLet).expect("Caller should have found this");
        let query = Query::parse(parser)?;
        parser
            .expect(OpComma)
            .map_err(|token| parser.expected(token, "expected `,` to follow `let` expression"))?;
        let body = Expression::parse_precedence(parser, Precedence::Sequence)?;
        Ok(Self { start, query, body })
    }
}
