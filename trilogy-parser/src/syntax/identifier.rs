use super::*;
use crate::{Parser, Spanned};
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
}

impl TryFrom<ModuleReference> for Identifier {
    type Error = SyntaxError;

    fn try_from(value: ModuleReference) -> Result<Self, Self::Error> {
        if value.arguments.is_none() {
            Ok(value.name)
        } else {
            Err(SyntaxError::new(
                value.span(),
                "identifiers may not have arguments",
            ))
        }
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
