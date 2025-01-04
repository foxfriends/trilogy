use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ReturnExpression {
    pub r#return: Token,
    pub expression: Expression,
    span: Span,
}

impl Spanned for ReturnExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl ReturnExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#return = parser.expect(KwReturn).unwrap();
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: r#return.span.union(expression.span()),
            r#return,
            expression,
        })
    }
}
