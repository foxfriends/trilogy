use super::*;

#[derive(Clone, Debug)]
pub struct Branch {
    pub condition: Vec<Code>,
    pub body: Vec<Code>,
}
