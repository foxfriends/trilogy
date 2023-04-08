use super::{pattern::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct NegativePattern {
    start: Token,
    pub pattern: Pattern,
}

impl NegativePattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OpMinus)
            .expect("Caller should have found this");
        let pattern = Pattern::parse_precedence(parser, Precedence::Unary)?;
        Ok(Self { start, pattern })
    }

    pub fn minus_token(&self) -> &Token {
        &self.start
    }
}
