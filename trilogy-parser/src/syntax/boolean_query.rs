use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BooleanQuery {
    start: Token,
    pub expression: Expression,
}

impl BooleanQuery {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser.expect(KwIs).expect("Caller should have found this");
        let expression = Expression::parse_parameter_list(parser)?; // this isn't a parameter list, but we don't allow commas
        Ok(Self { start, expression })
    }
}
