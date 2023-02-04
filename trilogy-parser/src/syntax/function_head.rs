use super::*;

#[derive(Clone, Debug)]
pub struct FunctionHead {
    pub name: Identifier,
    pub parameters: Vec<Pattern>,
}
