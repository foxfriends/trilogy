use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BooleanLiteral {
    token: Token,
}

impl BooleanLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect([TokenType::KwTrue, TokenType::KwFalse])
            .map_err(|token| parser.expected(token, "expected boolean literal"))?;
        Ok(Self { token })
    }

    pub fn value(&self) -> bool {
        match self.token.token_type {
            TokenType::KwTrue => true,
            TokenType::KwFalse => false,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(bool_true: "true" => BooleanLiteral::parse => "(BooleanLiteral)");
    test_parse!(bool_false: "false" => BooleanLiteral::parse => "(BooleanLiteral)");
    test_parse_error!(not_bool: "unit" => BooleanLiteral::parse => "expected boolean literal");
}
