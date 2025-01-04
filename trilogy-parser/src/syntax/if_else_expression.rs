use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct IfElseExpression {
    pub r#if: Token,
    pub condition: Expression,
    pub when_true: IfBody,
    pub when_false: Option<ElseClause>,
}

impl Spanned for IfElseExpression {
    fn span(&self) -> Span {
        match &self.when_false {
            Some(case) => case.span().union(self.r#if.span),
            None => self.when_true.span().union(self.r#if.span),
        }
    }
}

impl IfElseExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#if = parser.expect(KwIf).expect("Caller should have found this");
        let condition = Expression::parse(parser)?;
        let when_true = IfBody::parse(parser)?;

        let when_false = if parser.check(KwElse).is_ok() {
            Some(ElseClause::parse(parser)?)
        } else {
            None
        };

        Ok(Self {
            r#if,
            condition,
            when_true,
            when_false,
        })
    }

    pub(crate) fn strict_statement(self) -> SyntaxResult<Self> {
        if self.is_strict_statement() {
            return Ok(self);
        }
        if self.is_strict_expression() {
            return Ok(self);
        }
        Err(ErrorKind::IfStatementRestriction.at(self.span()))
    }

    pub(crate) fn strict_expression(self) -> SyntaxResult<Self> {
        if self.is_strict_expression() {
            return Ok(self);
        }
        Err(ErrorKind::IfExpressionRestriction.at(self.span()))
    }

    fn is_strict_statement(&self) -> bool {
        if !matches!(self.when_true, IfBody::Block(..)) {
            return false;
        }
        match &self.when_false {
            Some(case) => match &case.expression {
                Expression::IfElse(else_if) => else_if.is_strict_statement(),
                _ => false,
            },
            _ => true,
        }
    }

    fn is_strict_expression(&self) -> bool {
        self.when_false.is_some()
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum IfBody {
    Then(Token, Expression),
    Block(Block),
}

impl IfBody {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if parser.check(KwThen).is_ok() {
            let then = parser.expect(KwThen).unwrap();
            let body = Expression::parse(parser)?;
            Ok(Self::Then(then, body))
        } else {
            Ok(Self::Block(Block::parse(parser)?))
        }
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ElseClause {
    pub r#else: Token,
    pub expression: Expression,
}

impl ElseClause {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#else = parser
            .expect(KwElse)
            .expect("Caller should have found this");
        let expression = Expression::parse(parser)?;
        Ok(Self { r#else, expression })
    }
}
