use super::{Atom, Value};
use std::fmt::{self, Display};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Struct {
    name: Atom,
    value: Box<Value>,
}

impl Struct {
    pub fn new(name: Atom, value: Value) -> Self {
        Self {
            name,
            value: Box::new(value),
        }
    }

    pub fn name(&self) -> Atom {
        self.name.clone()
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn destruct(self) -> (Atom, Value) {
        (self.name, *self.value)
    }

    pub fn into_value(self) -> Value {
        *self.value
    }
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
