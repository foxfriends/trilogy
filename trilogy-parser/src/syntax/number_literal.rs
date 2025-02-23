use super::*;
use crate::Parser;
use num::{Complex, rational::BigRational};
use trilogy_scanner::{Token, TokenType, TokenValue};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct NumberLiteral {
    token: Token,
}

impl NumberLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Numeric)
            .map_err(|token| parser.expected(token, "expected number literal"))?;
        Ok(Self { token })
    }

    pub fn value(&self) -> Complex<BigRational> {
        let TokenValue::Number(number) = self.token.value.as_ref().unwrap() else {
            unreachable!()
        };
        *number.clone()
    }
}
