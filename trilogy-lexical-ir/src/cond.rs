use super::*;

#[derive(Clone, Debug)]
pub struct Cond {
    pub cond: Vec<Code>,
    pub body: Vec<Code>,
    pub is_loop: bool,
}

impl Cond {
    pub fn new(cond: Vec<Code>, body: Vec<Code>) -> Self {
        Self {
            cond,
            body,
            is_loop: false,
        }
    }

    pub fn new_loop(cond: Vec<Code>, body: Vec<Code>) -> Self {
        Self {
            cond,
            body,
            is_loop: true,
        }
    }
}
