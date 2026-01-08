use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub enum CallConv {
    Trilogy,
    C,
}

#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    pub name: Identifier,
    pub call_conv: CallConv,
    pub arity: usize,
    pub overloads: Vec<Procedure>,
}

impl ProcedureDefinition {
    pub(super) fn declare(name: Identifier, arity: usize) -> Self {
        Self {
            name,
            arity,
            call_conv: CallConv::Trilogy,
            overloads: vec![],
        }
    }

    pub(super) fn declare_extern(name: Identifier, call_conv: CallConv, arity: usize) -> Self {
        Self {
            name,
            arity,
            call_conv,
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
