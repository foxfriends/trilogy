use super::*;

#[derive(Clone, Debug)]
pub struct SinglePattern {
    pub value_pattern: ValuePattern,
    pub type_pattern: Option<TypePattern>,
}
