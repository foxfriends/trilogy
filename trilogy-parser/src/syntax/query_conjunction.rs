use super::{query::Precedence, *};
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct QueryConjunction {
    pub lhs: Query,
    pub rhs: Query,
}

impl QueryConjunction {
    pub(crate) fn parse(parser: &mut Parser, lhs: Query) -> SyntaxResult<Self> {
        parser.expect(KwAnd).expect("Caller should have found this");
        let rhs = Query::parse_precedence(parser, Precedence::Conjunction)?;
        Ok(Self { lhs, rhs })
    }
}
