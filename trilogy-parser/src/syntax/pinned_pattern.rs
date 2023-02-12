use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PinnedPattern {
    start: Token,
    pub identifier: Identifier,
}

impl PinnedPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::OpCaret)
            .expect("Caller should have found this");
        let identifier = Identifier::parse(parser)?;
        Ok(Self { start, identifier })
    }
}
