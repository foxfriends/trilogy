use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct IfElseExpression {
    pub r#if: Token,
    pub condition: Expression,
    pub when_true: FollowingExpression,
    pub when_false: Option<ElseClause>,
    pub span: Span,
}

impl Spanned for IfElseExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl IfElseExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#if = parser.expect(KwIf).expect("Caller should have found this");
        let condition = Expression::parse(parser)?;
        let when_true = FollowingExpression::parse(parser)?;

        let when_false = if parser.check(KwElse).is_ok() {
            Some(ElseClause::parse(parser)?)
        } else {
            None
        };

        let span = match &when_false {
            Some(case) => case.span().union(r#if.span),
            None => when_true.span().union(r#if.span),
        };

        Ok(Self {
            r#if,
            condition,
            when_true,
            when_false,
            span,
        })
    }

    pub(crate) fn is_strict_expression(&self) -> bool {
        self.when_false.is_some()
    }
}

#[derive(Clone, Debug, Spanned)]
pub struct ElseClause {
    pub r#else: Token,
    pub body: Expression,
}

impl ElseClause {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#else = parser
            .expect(KwElse)
            .expect("Caller should have found this");
        let body = Expression::parse(parser)?;
        Ok(Self { r#else, body })
    }
}
