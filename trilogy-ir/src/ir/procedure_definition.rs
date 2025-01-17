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
        self.overloads
            .first()
            .unwrap()
            .span
            .union(self.overloads.last().unwrap().span)
    }
}
