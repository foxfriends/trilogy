use super::{Atom, Value};
use std::fmt::{self, Display};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Struct {
    pub name: Atom,
    pub value: Box<Value>,
}

impl PartialOrd for Struct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.name == other.name {
            (*self.value).partial_cmp(&*other.value)
        } else {
            None
        }
    }
}

impl Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name, self.value)
    }
}
