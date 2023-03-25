use super::*;

#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    pub name: Identifier,
    pub overloads: Vec<Procedure>,
}

impl ProcedureDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self {
            name,
            overloads: vec![],
        }
    }
}
