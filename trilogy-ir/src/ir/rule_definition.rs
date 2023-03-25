use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct RuleDefinition {
    span: Span,
    pub parameters: Vec<Pattern>,
    pub body: Query,
}
