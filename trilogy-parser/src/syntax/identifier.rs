use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Identifier {
    token: Token,
}

impl Identifier {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Identifier)
            .map_err(|token| parser.expected(token, "expected identifier"))?;
        Ok(Self { token })
    }

    pub(crate) fn parse_eq(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::IdentifierEq)
            .map_err(|token| parser.expected(token, "expected assignment identifier"))?;
        Ok(Self { token })
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.token.value.as_ref().unwrap().as_str().unwrap()
    }
}

impl From<Identifier> for String {
    fn from(identifier: Identifier) -> String {
        identifier.token.value.unwrap().try_into().unwrap()
    }
}

impl TryFrom<Pattern> for Identifier {
    type Error = Pattern;

    fn try_from(value: Pattern) -> Result<Self, Pattern> {
        match value {
            Pattern::Binding(binding) if binding.is_immutable() => Ok(binding.identifier),
            _ => Err(value),
        }
    }
}
