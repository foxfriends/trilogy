use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct UnitLiteral {
    pub span: Span,
    pub token: Token,
}

impl Spanned for UnitLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl UnitLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::KwUnit)
            .map_err(|token| parser.expected(token, "expected boolean literal"))?;
        Ok(Self {
            span: token.span,
            token,
        })
    }
}
