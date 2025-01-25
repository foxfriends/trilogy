use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    pub name: Identifier,
    pub arity: usize,
    pub overloads: Vec<Procedure>,
}

impl ProcedureDefinition {
    pub(super) fn declare(name: Identifier, arity: usize) -> Self {
        Self {
            name,
            arity,
            overloads: vec![],
        }
    }

    pub fn span(&self) -> Span {
        match self.overloads.as_slice() {
            [first] => first.head_span.union(first.span),
            [] => self.name.span,
            _ => unreachable!("prodecure can have only one overload"),
        }
    }
}
