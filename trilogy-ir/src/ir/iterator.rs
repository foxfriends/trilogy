use super::*;

#[derive(Clone, Debug)]
pub struct Iterator {
    pub value: Expression,
    pub query: Expression,
}

impl Iterator {
    pub(super) fn new(query: Expression, value: Expression) -> Self {
        Self { value, query }
    }
}
