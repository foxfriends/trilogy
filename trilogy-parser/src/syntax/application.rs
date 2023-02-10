use super::{value_expression::Precedence, *};
use crate::Parser;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Application {
    pub function: Expression,
    pub argument: Expression,
}

impl Application {
    pub(crate) fn parse(parser: &mut Parser, lhs: impl Into<Expression>) -> SyntaxResult<Self> {
        let argument = ValueExpression::parse_precedence(parser, Precedence::Application)?;
        Ok(Self {
            function: lhs.into(),
            argument: argument.into(),
        })
    }
}
