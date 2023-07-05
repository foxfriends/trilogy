use std::fmt::Display;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Procedure(usize);

impl Procedure {
    pub(crate) fn new(pointer: usize) -> Self {
        Self(pointer)
    }

    pub(crate) fn ip(&self) -> usize {
        self.0
    }
}

impl Display for Procedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&({})", self.0)
    }
}
