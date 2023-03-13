use super::*;

#[derive(Clone, Debug)]
pub enum Violation {
    RuntimeTypeError,
    AssertionError(Evaluation),
    Exit(Evaluation),
}
