use super::{pattern::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TuplePattern {
    pub lhs: Pattern,
    cons_token: Token,
    pub rhs: Pattern,
}

impl TuplePattern {
    pub(crate) fn parse(parser: &mut Parser, lhs: Pattern) -> SyntaxResult<Self> {
        let cons_token = parser
            .expect(OpColon)
            .expect("Caller should have found this");
        let rhs = Pattern::parse_precedence(parser, Precedence::Cons)?;
        Ok(Self {
            lhs,
            cons_token,
            rhs,
        })
    }

    pub fn cons_token(&self) -> &Token {
        &self.cons_token
    }
}
