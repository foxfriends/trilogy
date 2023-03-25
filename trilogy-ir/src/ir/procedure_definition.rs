use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    span: Span,
    pub parameters: Vec<Pattern>,
    pub body: Expression,
}
