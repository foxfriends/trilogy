use super::*;

#[derive(Clone, Debug)]
pub struct Expression {
    pub expression: ValueExpression,
    pub handlers: Vec<Handler>,
}
