use crate::Parser;
use trilogy_scanner::{
    Token,
    TokenType::{self, DocInner, DocOuter},
};

#[derive(Clone, Debug, Spanned)]
pub struct Documentation {
    tokens: Vec<Token>,
}

impl Documentation {
    fn parse(parser: &mut Parser, token_type: TokenType) -> Option<Self> {
        let mut tokens = vec![];

        while let Ok(token) = parser.expect(token_type) {
            tokens.push(token);
        }
        if tokens.is_empty() {
            return None;
        }

        Some(Self { tokens })
    }

    pub(crate) fn parse_inner(parser: &mut Parser) -> Option<Self> {
        Self::parse(parser, DocInner)
    }

    pub(crate) fn parse_outer(parser: &mut Parser) -> Option<Self> {
        Self::parse(parser, DocOuter)
    }
}
