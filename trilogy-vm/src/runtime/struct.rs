use super::{Atom, Value};

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Struct {
    pub name: Atom,
    pub value: Value,
}
