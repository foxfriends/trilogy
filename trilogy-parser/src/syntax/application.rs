use super::{expression::Precedence, *};
use crate::Parser;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Application {
    pub function: Expression,
    pub argument: Expression,
}

impl Application {
    pub(crate) fn parse(parser: &mut Parser, function: Expression) -> SyntaxResult<Self> {
        let argument = Expression::parse_precedence(parser, Precedence::Application)?;
        Ok(Self { function, argument })
    }
}
