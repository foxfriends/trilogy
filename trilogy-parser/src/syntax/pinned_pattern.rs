use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct PinnedPattern {
    pub pin: Token,
    pub identifier: Identifier,
    pub span: Span,
}

impl Spanned for PinnedPattern {
    fn span(&self) -> Span {
        self.span
    }
}

impl PinnedPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let pin = parser.expect(TokenType::OpCaret).unwrap();
        let identifier = Identifier::parse(parser)?;
        Ok(Self {
            span: pin.span.union(identifier.span),
            pin,
            identifier,
        })
    }
}
