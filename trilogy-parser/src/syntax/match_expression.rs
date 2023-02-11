use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct MatchExpression {
    start: Token,
    pub expression: Expression,
    pub cases: Vec<MatchExpressionCase>,
    pub no_match: Expression,
}

impl MatchExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<MatchExpression> {
        let start = parser
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
        parser.expect(KwElse).map_err(|token| {
            parser.expected(token, "expected `else` case to end a `match` expression")
        })?;
        let no_match = Expression::parse_precedence(parser, Precedence::Continuation)?;

        Ok(Self {
            start,
            expression,
            cases,
            no_match,
        })
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchExpressionCase {
    start: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Expression,
}

impl MatchExpressionCase {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
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
            start,
            pattern,
            guard,
            body,
        })
    }
}

impl Spanned for MatchExpressionCase {
    fn span(&self) -> Span {
        self.start.span.union(self.body.span())
    }
}
