use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchExpression {
    pub r#match: Token,
    pub expression: Expression,
    pub obrace: Token,
    pub cases: Vec<MatchExpressionCase>,
    pub else_case: Option<MatchExpressionElseCase>,
    pub cbrace: Token,
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

        let obrace = parser
            .expect(OBrace)
            .map_err(|token| parser.expected(token, "expected { to begin `match` body"))?;
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

        let cbrace = parser
            .expect(CBrace)
            .map_err(|token| parser.expected(token, "expected } to end `match` body"))?;

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
            obrace,
            cases,
            else_case,
            cbrace,
        })
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchExpressionCase {
    pub case: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Guard>,
    pub body: FollowingExpression,
    span: Span,
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

        let body = FollowingExpression::parse(parser)?;
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
    pub body: Expression,
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
        let body = Expression::parse(parser)?;
        Ok(Self {
            span: r#else.span.union(body.span()),
            r#else,
            body,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(match_parse_exprs: "match x { case 1 then x\n  else y }" => MatchExpression::parse => "
      (MatchExpression
        _
        (Expression::Reference _)
        _
        [(MatchExpressionCase _ _ _ _)]
        (MatchExpressionElseCase _ _)
        _)
    ");
    test_parse_error!(match_no_parse_else_not_last: "match x { case 1 then x\n  else y\n  case 2 then x }" => MatchExpression::parse);
}
