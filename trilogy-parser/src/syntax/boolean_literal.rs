use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// A boolean literal expression.
///
/// ```trilogy
/// true
/// ```
#[derive(Clone, Debug)]
pub struct BooleanLiteral {
    pub token: Token,
    pub span: Span,
}

impl Spanned for BooleanLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl BooleanLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect([TokenType::KwTrue, TokenType::KwFalse])
            .map_err(|token| parser.expected(token, "expected boolean literal"))?;
        Ok(Self {
            span: token.span,
            token,
        })
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

    test_parse!(bool_true: "true" => BooleanLiteral::parse => BooleanLiteral { .. });
    test_parse!(bool_false: "false" => BooleanLiteral::parse => BooleanLiteral { .. });
    test_parse_error!(not_bool: "unit" => BooleanLiteral::parse => "expected boolean literal");
}
