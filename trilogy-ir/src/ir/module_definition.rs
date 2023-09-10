use super::*;
use once_cell::sync::OnceCell;
use std::sync::Arc;

#[derive(Debug)]
pub enum ModuleCell {
    Module(OnceCell<Module>),
    External(String),
}

impl ModuleCell {
    pub fn new(module: Module) -> Self {
        Self::Module(OnceCell::with_value(module))
    }

    pub fn insert(&self, module: Module) {
        match self {
            ModuleCell::Module(cell) => cell.set(module).unwrap(),
            ModuleCell::External(..) => panic!(),
        }
    }

    pub fn as_module(&self) -> Option<&Module> {
        match self {
            ModuleCell::Module(cell) => cell.get(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModuleDefinition {
    pub name: Identifier,
    pub module: Arc<ModuleCell>,
}

impl ModuleDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self {
            name,
            module: Arc::new(ModuleCell::Module(OnceCell::default())),
        }
    }

    pub(super) fn external(name: Identifier, location: String) -> Self {
        Self {
            name,
            module: Arc::new(ModuleCell::External(location)),
        }
    }
}
