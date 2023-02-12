use super::{pattern::Precedence, *};
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PatternDisjunction {
    pub lhs: Pattern,
    pub rhs: Pattern,
}

impl PatternDisjunction {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        parser.expect(KwOr).expect("Caller should have found this");
        let rhs = Pattern::parse_precedence(parser, Precedence::Disjunction)?;
        Ok(Self { lhs, rhs })
    }
}
