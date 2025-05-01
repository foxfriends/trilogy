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
    span: Span,
}

impl Spanned for MatchExpression {
    fn span(&self) -> Span {
        self.span
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

        let span = match &else_case {
            Some(case) => r#match.span.union(case.span()),
            None => r#match.span.union(
                cases
                    .last()
                    .map_or_else(|| expression.span(), Spanned::span),
            ),
        };

        Ok(Self {
            span,
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
    pub guard: Option<Guard>,
    pub body: MatchExpressionCaseBody,
    span: Span,
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum MatchExpressionCaseBody {
    Then(Token, Expression),
    Block(Block),
}

impl MatchExpressionCaseBody {
    fn parse(parser: &mut Parser, precedence: expression::Precedence) -> SyntaxResult<Self> {
        if let Ok(token) = parser.expect(KwThen) {
            let body = Expression::parse_precedence(parser, precedence)?;
            Ok(MatchExpressionCaseBody::Then(token, body))
        } else {
            Ok(MatchExpressionCaseBody::Block(Block::parse(parser)?))
        }
    }
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
        let guard = Guard::parse_optional(parser)?;
        let body = MatchExpressionCaseBody::parse(parser, Precedence::None)?;
        Ok(Self {
            span: case.span.union(body.span()),
            case,
            pattern,
            guard,
            body,
        })
    }
}

impl Spanned for MatchExpressionCase {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Guard {
    pub r#if: Token,
    pub expression: Expression,
    span: Span,
}

impl Spanned for Guard {
    fn span(&self) -> Span {
        self.span
    }
}

impl Guard {
    fn parse_optional(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        let Ok(r#if) = parser.expect(KwIf) else {
            return Ok(None);
        };
        let expression = Expression::parse(parser)?;
        Ok(Some(Self {
            span: r#if.span.union(expression.span()),
            r#if,
            expression,
        }))
    }
}
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchExpressionElseCase {
    pub r#else: Token,
    pub else_binding: Option<Pattern>,
    pub body: MatchExpressionCaseBody,
    span: Span,
}

impl Spanned for MatchExpressionElseCase {
    fn span(&self) -> Span {
        self.span
    }
}

impl MatchExpressionElseCase {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#else = parser.expect(KwElse).unwrap();
        let else_binding = if parser.check(Discard).is_ok() {
            Some(Pattern::parse(parser)?)
        } else if parser.check(KwThen).is_ok() || parser.check(OBrace).is_ok() {
            None
        } else {
            Some(Pattern::Binding(Box::new(BindingPattern::parse(parser)?)))
        };
        let body = MatchExpressionCaseBody::parse(parser, Precedence::Continuation)?;
        Ok(Self {
            span: r#else.span.union(body.span()),
            r#else,
            else_binding,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(match_parse_exprs: "match x\n  case 1 then x\n  else then y" => MatchExpression::parse => "
      (MatchExpression
        _
        (Expression::Reference _)
        [(MatchExpressionCase _ _ _ _)]
        (MatchExpressionElseCase _ _ _))
    ");
}
