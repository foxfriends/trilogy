use super::*;

#[derive(Clone, Debug)]
pub struct Application {
    pub function: Expression,
    pub argument: Expression,
}

impl Application {
    pub(super) fn new(function: Expression, argument: Expression) -> Self {
        Self { function, argument }
    }
}
