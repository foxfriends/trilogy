use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ResumeExpression {
    start: Token,
    pub expression: Expression,
}

impl ResumeExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwResume)
            .expect("Caller should have found this");
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self { start, expression })
    }

    pub fn resume_token(&self) -> &Token {
        &self.start
    }
}
