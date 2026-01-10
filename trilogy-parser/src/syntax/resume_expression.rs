use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct ResumeExpression {
    pub resume: Token,
    pub expression: Expression,
    pub span: Span,
}

impl Spanned for ResumeExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl ResumeExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let resume = parser.expect(KwResume).unwrap();
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: resume.span.union(expression.span()),
            resume,
            expression,
        })
    }
}
