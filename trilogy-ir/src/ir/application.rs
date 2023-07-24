use super::*;
use crate::Id;

#[derive(Clone, Debug)]
pub struct Application {
    pub function: Expression,
    pub argument: Expression,
}

impl Application {
    pub(super) fn new(function: Expression, argument: Expression) -> Self {
        Self { function, argument }
    }

    pub fn bindings(&self) -> Box<dyn std::iter::Iterator<Item = Id> + '_> {
        match self.function.value {
            Value::Builtin(Builtin::Pin) => Box::new(std::iter::empty()),
            _ => Box::new(self.function.bindings().chain(self.argument.bindings())),
        }
    }
}
