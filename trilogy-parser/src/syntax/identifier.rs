use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Eq, PartialEq, Debug)]
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

impl Spanned for Identifier {
    fn span(&self) -> Span {
        self.token.span
    }
}
