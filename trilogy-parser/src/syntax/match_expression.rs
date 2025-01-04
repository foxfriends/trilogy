use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchExpression {
    pub r#match: Token,
    pub expression: Expression,
    pub cases: Vec<MatchExpressionCase>,
    pub else_case: Option<MatchExpressionElseCase>,
}

impl Spanned for MatchExpression {
    fn span(&self) -> Span {
        match &self.else_case {
            Some(case) => self.r#match.span.union(case.span()),
            None => self.r#match.span.union(
                self.cases
                    .last()
                    .map_or_else(|| self.expression.span(), Spanned::span),
            ),
        }
    }
}

impl MatchExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<MatchExpression> {
        let r#match = parser
            .expect(KwMatch)
            .expect("Caller should have found this");

        let expression = Expression::parse(parser)?;

        let mut cases = vec![];
        loop {
            if let Err(token) = parser.check(KwCase) {
                let error = SyntaxError::new(
                    token.span,
                    "expected at least one `case` to follow a `match` body",
                );
                parser.error(error.clone());
                return Err(error);
            }
            cases.push(MatchExpressionCase::parse(parser)?);
            if parser.check(KwCase).is_err() {
                break;
            }
        }

        let else_case = if parser.check(KwElse).is_ok() {
            Some(MatchExpressionElseCase::parse(parser)?)
        } else {
            None
        };

        Ok(Self {
            r#match,
            expression,
            cases,
            else_case,
        })
    }

    pub(crate) fn strict_expression(self) -> SyntaxResult<Self> {
        if self.else_case.is_some() {
            return Ok(self);
        }
        Err(ErrorKind::MatchExpressionRestriction.at(self.span()))
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchExpressionCase {
    pub case: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Expression,
}

impl MatchExpressionCase {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let case = parser
            .expect(KwCase)
            .expect("Caller should have found this");
        let pattern = parser
            .expect(KwIf)
            .err()
            .map(|_| Pattern::parse(parser))
            .transpose()?;
        let guard = parser
            .expect(KwIf)
            .ok()
            .map(|_| Expression::parse(parser))
            .transpose()?
            .map(Into::into);
        parser.expect(KwThen).map_err(|token| {
            parser.expected(token, "expected `then` to mark the body of a `case`")
        })?;
        let body = Expression::parse(parser)?;
        Ok(Self {
            case,
            pattern,
            guard,
            body,
        })
    }
}

impl Spanned for MatchExpressionCase {
    fn span(&self) -> Span {
        self.case.span.union(self.body.span())
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct MatchExpressionElseCase {
    pub r#else: Token,
    pub else_binding: Pattern,
    pub no_match: Expression,
}

impl MatchExpressionElseCase {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#else = parser
            .expect(KwElse)
            .expect("caller should have found this");
        let else_binding = if parser.check(Discard).is_ok() {
            Pattern::parse(parser)?
        } else {
            Pattern::Binding(Box::new(BindingPattern::parse(parser)?))
        };
        parser.expect(KwThen).map_err(|token| {
            parser.expected(token, "expected `then` keyword to follow else case")
        })?;
        let no_match = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            r#else,
            else_binding,
            no_match,
        })
    }
}
