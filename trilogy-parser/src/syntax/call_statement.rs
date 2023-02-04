use super::*;

#[derive(Clone, Debug)]
pub struct CallStatement {
    pub call: CallExpression,
    pub handlers: Vec<Handler>,
}
