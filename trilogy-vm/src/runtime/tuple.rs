use std::fmt::{self, Display};

use super::Value;

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Hash)]
pub struct Tuple(Box<(Value, Value)>);

impl Tuple {
    pub fn new(lhs: Value, rhs: Value) -> Self {
        Self(Box::new((lhs, rhs)))
    }
}

impl Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}:{})", (self.0).0, (self.0).1)
    }
}
