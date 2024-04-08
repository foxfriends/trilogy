use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchStatement {
    pub r#match: Token,
    pub expression: Expression,
    pub cases: Vec<MatchStatementCase>,
    pub else_case: Option<ElseCase>,
}

impl MatchStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#match = parser
            .expect(KwMatch)
            .expect("Caller should have found this");
        let expression = Expression::parse(parser)?;
        let mut cases = vec![];
        while parser.check(KwCase).is_ok() {
            cases.push(MatchStatementCase::parse(parser)?);
        }
        if cases.is_empty() {
            let error =
                SyntaxError::new(r#match.span, "match statement must have at least one case");
            parser.error(error);
        }
        let else_case = if parser.check(KwElse).is_ok() {
            Some(ElseCase::parse(parser)?)
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
}

impl Spanned for MatchStatement {
    fn span(&self) -> Span {
        match &self.else_case {
            None => self.r#match.span.union(self.cases.span()),
            Some(case) => self.r#match.span.union(case.span()),
        }
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct MatchStatementCase {
    pub case: Token,
    pub pattern: Option<Pattern>,
    pub guard: Option<Expression>,
    pub body: Block,
}

impl MatchStatementCase {
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
            case,
            pattern,
            guard,
            body,
        })
    }
}

impl Spanned for MatchStatementCase {
    fn span(&self) -> Span {
        self.case.span.union(self.body.span())
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ElseCase {
    pub r#else: Token,
    pub pattern: Option<BindingPattern>,
    pub body: Block,
}

impl ElseCase {
    fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#else = parser
            .expect(KwElse)
            .expect("Caller should have found this");
        let pattern = if parser.check([OBrace, KwThen]).is_err() {
            Some(BindingPattern::parse(parser)?)
        } else {
            None
        };
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
            r#else,
            pattern,
            body,
        })
    }
}

impl Spanned for ElseCase {
    fn span(&self) -> Span {
        self.r#else.span.union(self.body.span())
    }
}
