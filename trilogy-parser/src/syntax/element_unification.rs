use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ElementUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}

impl ElementUnification {
    pub(crate) fn parse(parser: &mut Parser, pattern: Pattern) -> SyntaxResult<Self> {
        parser.expect(KwIn).expect("Caller should have found this");
        let expression = Expression::parse_parameter_list(parser)?;
        Ok(Self {
            pattern,
            expression,
        })
    }
}
