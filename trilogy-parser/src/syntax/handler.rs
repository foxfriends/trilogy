use crate::Parser;

use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Handler {
    Given(Box<GivenHandler>),
    When(Box<WhenHandler>),
    Else(Box<ElseHandler>),
}

impl Handler {
    pub(crate) fn parse(_parser: &mut Parser) -> SyntaxResult<Self> {
        todo!()
    }
}
