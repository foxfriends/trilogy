use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct Identifier {
    pub token: Token,
    pub span: Span,
}

impl Spanned for Identifier {
    fn span(&self) -> Span {
        self.span
    }
}

impl Identifier {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Identifier)
            .map_err(|token| parser.expected(token, "expected identifier"))?;
        Ok(Self {
            span: token.span,
            token,
        })
    }

    pub(crate) fn parse_eq(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::IdentifierEq)
            .map_err(|token| parser.expected(token, "expected assignment identifier"))?;
        Ok(Self {
            span: token.span,
            token,
        })
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

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(identifier_normal: "hello" => Identifier::parse => Identifier { .. });
    test_parse!(identifier_underscored: "_hello" => Identifier::parse => Identifier { .. });
    test_parse!(identifier_numbers: "hello123" => Identifier::parse => Identifier { .. });
    test_parse_error!(identifier_keyword: "for" => Identifier::parse);

    test_parse!(identifiereq_normal: "hello=" => Identifier::parse_eq => Identifier { .. });
    test_parse!(identifiereq_underscored: "_hello=" => Identifier::parse_eq => Identifier { .. });
    test_parse!(identifiereq_numbers: "hello123=" => Identifier::parse_eq => Identifier { .. });
    test_parse_error!(identifiereq_keyword: "for=" => Identifier::parse_eq);
}
