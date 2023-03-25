use super::*;

#[derive(Clone, Debug)]
pub(super) struct ModuleDefinition {
    pub name: Identifier,
    pub module: Option<Module>,
}

impl ModuleDefinition {
    pub(crate) fn declare(name: Identifier) -> Self {
        Self { name, module: None }
    }
}
