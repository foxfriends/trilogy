use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug)]
pub struct StringLiteral {
    pub span: Span,
    pub token: Token,
}

impl Spanned for StringLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl StringLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::String)
            .map_err(|token| parser.expected(token, "expected string literal"))?;
        Ok(Self {
            span: token.span,
            token,
        })
    }

    pub fn value(&self) -> String {
        let TokenValue::String(value) = self.token.value.as_ref().unwrap() else {
            unreachable!()
        };
        value.clone()
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
