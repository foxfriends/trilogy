use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct BecomeExpression {
    pub r#become: Token,
    pub expression: Expression,
    span: Span,
}

impl Spanned for BecomeExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl BecomeExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#become = parser.expect(KwBecome).unwrap();
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: r#become.span.union(expression.span()),
            r#become,
            expression,
        })
    }
}
