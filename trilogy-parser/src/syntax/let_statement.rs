use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct LetStatement {
    pub r#let: Token,
    pub query: Query,
    span: Span,
}

impl Spanned for LetStatement {
    fn span(&self) -> Span {
        self.span
    }
}

impl LetStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#let = parser.expect(TokenType::KwLet).unwrap();
        let query = Query::parse_no_seq(parser)?;
        Ok(Self {
            span: r#let.span.union(query.span()),
            r#let,
            query,
        })
    }
}
