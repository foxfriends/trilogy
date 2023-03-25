use super::*;

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub overloads: Vec<Function>,
}

impl FunctionDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self {
            name,
            overloads: vec![],
        }
    }
}
