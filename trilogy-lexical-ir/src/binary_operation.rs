use super::*;

#[derive(Clone, Debug)]
pub struct BinaryOperation {
    pub lhs: Evaluation,
    pub rhs: Evaluation,
}

impl BinaryOperation {
    pub fn new(lhs: Evaluation, rhs: Evaluation) -> Self {
        Self { lhs, rhs }
    }
}
