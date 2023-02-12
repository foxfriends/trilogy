use super::{pattern::Precedence, *};
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct GluePattern {
    pub lhs: Pattern,
    pub rhs: Pattern,
}

impl GluePattern {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        parser
            .expect(OpGlue)
            .expect("Caller should have found this");
        let rhs = Pattern::parse_precedence(parser, Precedence::Glue)?;
        Ok(Self { lhs, rhs })
    }
}
