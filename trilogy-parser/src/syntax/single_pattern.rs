use super::*;
use crate::Spanned;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct SinglePattern {
    pub value_pattern: ValuePattern,
    pub type_pattern: Option<TypePattern>,
}

impl Spanned for SinglePattern {
    fn span(&self) -> Span {
        match &self.type_pattern {
            None => self.value_pattern.span(),
            Some(type_pattern) => self.value_pattern.span().union(type_pattern.span()),
        }
    }
}
