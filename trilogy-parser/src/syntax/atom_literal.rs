use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// An atom literal expression.
///
/// ```trilogy
/// 'atom
/// ```
#[derive(Clone, Debug)]
pub struct AtomLiteral {
    pub token: Token,
    pub span: Span,
}

impl Spanned for AtomLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl AtomLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Atom)
            .map_err(|token| parser.expected(token, "expected atom literal"))?;
        Ok(Self {
            span: token.span,
            token,
        })
    }

    /// The string value of this atom literal. Does not include the leading `'` character.
    pub fn value(&self) -> &str {
        self.token.value.as_ref().unwrap().as_str().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(atom: "'hello" => AtomLiteral::parse => AtomLiteral { .. });
    test_parse_error!(not_atom: "hello" => AtomLiteral::parse => "expected atom literal");
}
