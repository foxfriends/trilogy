use super::*;
use crate::Id;

#[derive(Clone, Debug)]
pub struct Iterator {
    pub value: Expression,
    pub query: Expression,
}

impl Iterator {
    pub(super) fn new(query: Expression, value: Expression) -> Self {
        Self { value, query }
    }

    pub fn bindings(&self) -> impl std::iter::Iterator<Item = Id> + '_ {
        self.value.bindings().chain(self.query.bindings())
    }
}
