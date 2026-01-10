use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// A binding pattern.
///
/// ```trilogy
/// mut x
/// ```
#[derive(Clone, Debug)]
pub struct BindingPattern {
    pub r#mut: Option<Token>,
    pub identifier: Identifier,
    pub span: Span,
}

impl BindingPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mutable = parser.expect(TokenType::KwMut).ok();
        let identifier = Identifier::parse(parser)?;
        Ok(Self {
            span: match &mutable {
                Some(mutable) => mutable.span.union(identifier.span),
                None => identifier.span,
            },
            r#mut: mutable,
            identifier,
        })
    }

    #[inline]
    pub fn is_immutable(&self) -> bool {
        self.r#mut.is_none()
    }

    #[inline]
    pub fn is_mutable(&self) -> bool {
        self.r#mut.is_some()
    }
}

impl Spanned for BindingPattern {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(binding_immutable: "hello" => BindingPattern::parse => BindingPattern { r#mut: None, .. });
    test_parse!(binding_mutable: "mut hello" => BindingPattern::parse => BindingPattern { r#mut: Some(..), .. });
    test_parse_error!(binding_not_name: "mut 'hello" => BindingPattern::parse => "expected identifier");
    test_parse_error!(binding_multiple: "mut hello, world" => BindingPattern::parse);

    #[test]
    fn test_is_immutable() {
        let binding = parse!("hello" => BindingPattern::parse);
        assert!(binding.is_immutable());
    }

    #[test]
    fn test_is_mutable() {
        let binding = parse!("mut hello" => BindingPattern::parse);
        assert!(!binding.is_immutable());
    }
}
