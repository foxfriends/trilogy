use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct StringLiteral {
    token: Token,
}

impl StringLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::String)
            .map_err(|token| parser.expected(token, "expected string literal"))?;
        Ok(Self { token })
    }
}

impl AsRef<str> for StringLiteral {
    fn as_ref(&self) -> &str {
        self.token.value.as_ref().unwrap().as_str().unwrap()
    }
}

impl From<StringLiteral> for String {
    fn from(literal: StringLiteral) -> String {
        literal.token.value.unwrap().try_into().unwrap()
    }
}
