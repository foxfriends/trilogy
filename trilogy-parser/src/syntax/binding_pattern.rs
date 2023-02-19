use super::*;
use crate::{Parser, Spanned};
use source_span::Span;

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct BindingPattern {
    pub mutable: MutModifier,
    pub identifier: Identifier,
}

impl BindingPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mutable = MutModifier::parse(parser);
        let identifier = Identifier::parse(parser)?;
        Ok(Self {
            mutable,
            identifier,
        })
    }

    pub(crate) fn is_immutable(&self) -> bool {
        matches!(self.mutable, MutModifier::Not)
    }
}

impl Spanned for BindingPattern {
    fn span(&self) -> Span {
        match &self.mutable {
            MutModifier::Not => self.identifier.span(),
            MutModifier::Mut(token) => token.span.union(self.identifier.span()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(binding_immutable: "hello" => BindingPattern::parse => "(BindingPattern () (Identifier))");
    test_parse!(binding_mutable: "mut hello" => BindingPattern::parse => "(BindingPattern (MutModifier::Mut _) (Identifier))");
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
