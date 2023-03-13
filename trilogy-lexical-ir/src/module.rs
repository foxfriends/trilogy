use super::*;
use source_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum EitherModule {
    Internal(Box<Module>),
    External(Box<ExternalModule>),
}

impl From<ExternalModule> for EitherModule {
    fn from(value: ExternalModule) -> Self {
        Self::External(Box::new(value))
    }
}

impl From<Module> for EitherModule {
    fn from(value: Module) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl EitherModule {
    pub fn span(&self) -> Span {
        match self {
            Self::Internal(module) => module.span,
            Self::External(module) => module.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub imported_modules: HashMap<Id, Evaluation>,
    pub imported_items: HashMap<Id, Evaluation>,
    pub submodules: HashMap<ItemKey, EitherModule>,
    pub items: HashMap<ItemKey, Vec<Item>>,
    pub tests: Vec<Test>,
    pub exported_items: HashMap<String, Export>,
}

#[derive(Clone, Debug)]
pub struct ExternalModule {
    pub span: Span,
    pub locator: String,
}
