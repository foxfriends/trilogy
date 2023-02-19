use super::Value;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct Tuple(Value, Value);
