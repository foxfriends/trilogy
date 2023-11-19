use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchStatement {
    start: Token,
    pub expression: Expression,
    pub cases: Vec<MatchStatementCase>,
    pub else_case: Option<Block>,
}

impl MatchStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwMatch)
            .expect("Caller should have found this");
        let expression = Expression::parse(parser)?;
        let mut cases = vec![];
        while parser.check(KwCase).is_ok() {
            cases.push(MatchStatementCase::parse(parser)?);
        }
        if cases.is_empty() {
            let error = SyntaxError::new(start.span, "match statement must have at least one case");
            parser.error(error);
        }
        let else_case = parser
            .expect(KwElse)
            .ok()
            .map(|_| Block::parse(parser))
            .transpose()?;
        Ok(Self {
            start,
            expression,
            cases,
            else_case,
        })
    }
}

impl Spanned for MatchStatement {
    fn span(&self) -> Span {
        match &self.else_case {
            None => self.start.span.union(self.cases.span()),
            Some(case) => self.start.span.union(case.span()),
        }
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchStatementCase {
    start: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Block,
}

impl MatchStatementCase {
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

        let body = match parser.expect(KwThen) {
            Ok(then) => {
                let error = ErrorKind::MatchStatementExpressionCase.at(then.span);
                parser.error(error.clone());
                Expression::parse(parser)?;
                return Err(error);
            }
            Err(_) => Block::parse(parser)?,
        };
        Ok(Self {
            start,
            pattern,
            guard,
            body,
        })
    }

    pub fn case_token(&self) -> &Token {
        &self.start
    }
}

impl Spanned for MatchStatementCase {
    fn span(&self) -> Span {
        self.start.span.union(self.body.span())
    }
}
