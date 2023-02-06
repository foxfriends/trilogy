use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct CallStatement {
    pub call: CallExpression,
    pub handlers: Vec<Handler>,
}
