use super::*;

#[derive(Clone, Debug)]
pub struct Collect {
    pub strategy: CollectStrategy,
    pub body: Vec<Code>,
    pub direction: Direction,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CollectStrategy {
    Void,   // For loop
    Scalar, // Existence
    Array,  // Comprehension
    Record,
    Set,
}

impl Collect {
    pub fn new_void(direction: Direction, body: Vec<Code>) -> Self {
        Self {
            body,
            direction,
            strategy: CollectStrategy::Void,
        }
    }

    pub fn new_scalar(direction: Direction) -> Self {
        Self {
            body: vec![],
            direction,
            strategy: CollectStrategy::Scalar,
        }
    }

    pub fn new_array(direction: Direction, body: Vec<Code>) -> Self {
        Self {
            body,
            direction,
            strategy: CollectStrategy::Array,
        }
    }

    pub fn new_record(direction: Direction, body: Vec<Code>) -> Self {
        Self {
            body,
            direction,
            strategy: CollectStrategy::Record,
        }
    }

    pub fn new_set(direction: Direction, body: Vec<Code>) -> Self {
        Self {
            body,
            direction,
            strategy: CollectStrategy::Set,
        }
    }
}
