use super::*;

#[derive(Clone, Debug)]
pub enum Violation {
    RuntimeTypeError,
    AssertionError(Box<Evaluation>),
    Exit(Box<Evaluation>),
}
