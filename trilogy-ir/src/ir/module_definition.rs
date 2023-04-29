use super::*;
use crate::Resolver;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(super) enum EitherModule {
    Reference(String),
    Module(Arc<Module>),
}

#[derive(Clone, Debug)]
pub struct ModuleDefinition {
    pub name: Identifier,
    pub(super) module: Option<EitherModule>,
}

impl ModuleDefinition {
    pub(super) fn declare(name: Identifier) -> Self {
        Self { name, module: None }
    }

    pub fn resolve<R>(&mut self, resolver: &mut R)
    where
        R: Resolver,
    {
        if let Some(EitherModule::Reference(path)) = &self.module {
            self.module = Some(EitherModule::Module(resolver.resolve(path)));
        }
    }
}
