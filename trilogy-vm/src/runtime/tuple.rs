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

impl<T, U> From<(T, U)> for Tuple
where
    Value: From<T>,
    Value: From<U>,
{
    fn from((t, u): (T, U)) -> Self {
        Self::new(t.into(), u.into())
    }
}
