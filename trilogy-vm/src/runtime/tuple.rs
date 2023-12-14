use super::{RefCount, ReferentialEq, StructuralEq, Value};
use std::fmt::{self, Display};

/// A Trilogy Tuple.
#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Hash)]
pub struct Tuple(RefCount<(Value, Value)>);

impl StructuralEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        StructuralEq::eq(&self.0 .0, &other.0 .0) && StructuralEq::eq(&self.0 .1, &other.0 .1)
    }
}

impl ReferentialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        ReferentialEq::eq(&self.0 .0, &other.0 .0) && ReferentialEq::eq(&self.0 .1, &other.0 .1)
    }
}

impl Tuple {
    #[inline]
    pub fn new(lhs: Value, rhs: Value) -> Self {
        Self(RefCount::new((lhs, rhs)))
    }

    #[inline]
    pub fn uncons(self) -> (Value, Value) {
        (*self.0).clone()
    }

    #[inline]
    pub fn first(&self) -> &Value {
        &(self.0).0
    }

    #[inline]
    pub fn into_first(self) -> Value {
        (*self.0).clone().0
    }

    #[inline]
    pub fn second(&self) -> &Value {
        &(self.0).1
    }

    #[inline]
    pub fn into_second(self) -> Value {
        (*self.0).clone().1
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
