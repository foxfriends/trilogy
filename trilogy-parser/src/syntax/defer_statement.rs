use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct DeferStatement {
    pub defer: Token,
    pub body: Block,
    span: Span,
}

impl DeferStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let defer = parser.expect(TokenType::KwDefer).unwrap();
        let body = Block::parse(parser)?;
        Ok(Self {
            span: defer.span.union(body.span()),
            defer,
            body,
        })
    }
}

impl Spanned for DeferStatement {
    fn span(&self) -> Span {
        self.span
    }
}
