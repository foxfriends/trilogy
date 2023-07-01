use super::Value;

#[derive(Clone, Eq, PartialEq, PartialOrd, Debug, Hash)]
pub struct Tuple(Box<(Value, Value)>);

impl Tuple {
    pub fn new(lhs: Value, rhs: Value) -> Self {
        Self(Box::new((lhs, rhs)))
    }
}
