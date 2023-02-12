use super::{pattern::Precedence, *};
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TuplePattern {
    pub lhs: Pattern,
    pub rhs: Pattern,
}

impl TuplePattern {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        parser
            .expect(OpColon)
            .expect("Caller should have found this");
        let rhs = Pattern::parse_precedence(parser, Precedence::Cons)?;
        Ok(Self { lhs, rhs })
    }
}
