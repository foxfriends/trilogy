use super::*;
use once_cell::sync::OnceCell;
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ModuleCell(OnceCell<Module>);

impl ModuleCell {
    pub fn new(module: Module) -> Self {
        Self(OnceCell::with_value(module))
    }

    pub fn insert(&self, module: Module) {
        self.0
            .set(module)
            .expect("module should not be inserted twice");
    }

    pub fn as_module(&self) -> Option<&Module> {
        self.0.get()
    }
}

#[derive(Clone, Debug)]
pub struct ModuleDefinition {
    pub name: Identifier,
    pub(crate) module: Arc<ModuleCell>,
}

impl ModuleDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self {
            name,
            module: Arc::new(ModuleCell::default()),
        }
    }
}
