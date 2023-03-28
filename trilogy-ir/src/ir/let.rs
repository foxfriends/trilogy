use super::*;

#[derive(Clone, Debug)]
pub struct Let {
    pub query: Query,
    pub body: Expression,
}

impl Let {
    pub(super) fn new(query: Query, body: Expression) -> Self {
        Self { query, body }
    }
}
