use super::*;

#[derive(Clone, Debug)]
pub struct Cond {
    pub cond: Vec<Code>,
    pub body: Vec<Code>,
}
