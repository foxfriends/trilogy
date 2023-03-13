use super::*;

#[derive(Clone, Debug)]
pub struct Call {
    pub func: Evaluation,
    pub args: Vec<Evaluation>,
}
