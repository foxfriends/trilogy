use crate::Value;
use std::fmt::{self, Display};

#[derive(Clone, Default, Debug)]
pub enum StackCell {
    #[default]
    Unset,
    Set(Value),
}

impl StackCell {
    pub fn into_set(self) -> Option<Value> {
        match self {
            StackCell::Unset => None,
            StackCell::Set(value) => Some(value),
        }
    }

    pub fn as_set(&self) -> Option<&Value> {
        match self {
            StackCell::Unset => None,
            StackCell::Set(value) => Some(value),
        }
    }

    pub fn is_set(&self) -> bool {
        matches!(self, StackCell::Set(..))
    }
}

impl Display for StackCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackCell::Unset => write!(f, "<unset>"),
            StackCell::Set(value) => write!(f, "{value}"),
        }
    }
}
