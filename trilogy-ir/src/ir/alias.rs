use super::*;

#[derive(Clone, Debug)]
pub struct Alias {
    pub name: Identifier,
    pub value: Option<Expression>,
}

impl Alias {
    pub(super) fn declare(name: Identifier) -> Self {
        Self { name, value: None }
    }
}
