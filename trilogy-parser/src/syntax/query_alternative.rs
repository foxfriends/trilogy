use super::{query::Precedence, *};
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct QueryAlternative {
    pub lhs: Query,
    pub rhs: Query,
}

impl QueryAlternative {
    pub(crate) fn parse(parser: &mut Parser, lhs: Query) -> SyntaxResult<Self> {
        parser
            .expect(KwElse)
            .expect("Caller should have found this");
        let rhs = Query::parse_precedence(parser, Precedence::Disjunction)?;
        Ok(Self { lhs, rhs })
    }
}
