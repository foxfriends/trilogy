use super::*;
use crate::{Parser, Spanned};
use num::{Complex, rational::BigRational};
use source_span::Span;
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug)]
pub struct NumberLiteral {
    pub token: Token,
    pub span: Span,
}

impl Spanned for NumberLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl NumberLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Numeric)
            .map_err(|token| parser.expected(token, "expected number literal"))?;
        Ok(Self {
            span: token.span,
            token,
        })
    }

    pub fn value(&self) -> Complex<BigRational> {
        let TokenValue::Number(number) = self.token.value.as_ref().unwrap() else {
            unreachable!()
        };
        *number.clone()
    }
}
