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

    pub fn bindings(&self) -> impl std::iter::Iterator<Item = Id> + '_ {
        self.function.bindings().chain(self.argument.bindings())
    }
}
