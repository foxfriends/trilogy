use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CallStatement {
    pub call: CallExpression,
    pub handlers: Vec<Handler>,
}
