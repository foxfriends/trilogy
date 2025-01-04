use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct LetExpression {
    pub r#let: Token,
    pub query: Query,
    pub body: Expression,
    span: Span,
}

impl Spanned for LetExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl LetExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#let = parser.expect(KwLet).unwrap();
        let query = Query::parse(parser)?;
        parser
            .expect(OpComma)
            .map_err(|token| parser.expected(token, "expected `,` to follow `let` expression"))?;
        let body = Expression::parse_precedence(parser, Precedence::Sequence)?;
        Ok(Self {
            span: r#let.span.union(body.span()),
            r#let,
            query,
            body,
        })
    }
}
