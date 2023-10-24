use crate::bytecode::Offset;
use std::fmt::{self, Display};
use std::hash::Hash;

/// A procedure from a Trilogy program.
///
/// From within the program this is seen as an opaque "callable" value.
///
/// It is not possible to construct a value of this type except from within a
/// Trilogy program.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Procedure(Offset);

impl Procedure {
    pub(crate) fn new(pointer: Offset) -> Self {
        Self(pointer)
    }

    pub(crate) fn ip(&self) -> Offset {
        self.0
    }
}

impl Display for Procedure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "&({})", self.0)
    }
}
