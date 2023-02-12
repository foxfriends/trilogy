use super::{pattern::Precedence, *};
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PatternConjunction {
    pub lhs: Pattern,
    pub rhs: Pattern,
}

impl PatternConjunction {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        parser.expect(KwAnd).expect("Caller should have found this");
        let rhs = Pattern::parse_precedence(parser, Precedence::Conjunction)?;
        Ok(Self { lhs, rhs })
    }
}
