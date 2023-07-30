use super::*;

#[derive(Clone, Debug)]
pub struct Iterator {
    pub value: Expression,
    pub query: Query,
}

impl Iterator {
    pub(super) fn new(query: Query, value: Expression) -> Self {
        Self { value, query }
    }
}
