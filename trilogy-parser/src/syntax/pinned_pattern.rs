use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PinnedPattern {
    pub pin: Token,
    pub identifier: Identifier,
}

impl PinnedPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let pin = parser.expect(TokenType::OpCaret).unwrap();
        let identifier = Identifier::parse(parser)?;
        Ok(Self { pin, identifier })
    }
}
