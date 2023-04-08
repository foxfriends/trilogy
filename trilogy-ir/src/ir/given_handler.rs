use super::*;
use source_span::Span;

// TODO: I might just delete `given` from the language, it has proven to be
// nothing but trouble. Probably needs to be reintroduced in a different way

#[derive(Clone, Debug)]
pub struct GivenHandler {
    pub span: Span,
    pub name: Identifier,
    pub rule: Rule,
}
