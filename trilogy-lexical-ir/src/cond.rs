use super::*;

#[derive(Clone, Debug)]
pub struct Cond {
    pub cond: Vec<Code>,
    pub body: Vec<Code>,
}

impl Cond {
    pub fn new(cond: Vec<Code>, body: Vec<Code>) -> Self {
        Self { cond, body }
    }
}
