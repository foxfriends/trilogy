use std::hash::Hash;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Procedure {
    ip: usize,
}

impl Procedure {
    pub fn ip(&self) -> usize {
        self.ip
    }
}
