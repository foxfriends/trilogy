use super::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(super) enum EitherModule {
    Reference(String),
    Module(Arc<Module>),
}

#[derive(Clone, Debug)]
pub(super) struct ModuleDefinition {
    pub name: Identifier,
    pub module: Option<EitherModule>,
}

impl ModuleDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self { name, module: None }
    }
}
