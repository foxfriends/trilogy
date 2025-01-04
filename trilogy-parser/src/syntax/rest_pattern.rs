use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RestPattern {
    pub rest: Token,
    pub pattern: Option<Pattern>,
    span: Span,
}

impl Spanned for RestPattern {
    fn span(&self) -> Span {
        self.span
    }
}

impl RestPattern {
    pub(crate) fn new(rest: Token, pattern: Pattern) -> Self {
        Self {
            span: rest.span.union(pattern.span()),
            rest,
            pattern: Some(pattern),
        }
    }

    pub(crate) fn parse(parser: &mut Parser, rest: Token) -> SyntaxResult<Self> {
        if parser
            .check([
                TokenType::OpComma,
                TokenType::CBrack,
                TokenType::CBracePipe,
                TokenType::CBrackPipe,
            ])
            .is_ok()
        {
            return Ok(Self {
                span: rest.span,
                rest,
                pattern: None,
            });
        }
        let pattern = Pattern::parse(parser)?;
        Ok(Self {
            span: rest.span.union(pattern.span()),
            rest,
            pattern: Some(pattern),
        })
    }
}

impl TryFrom<(Token, Expression)> for RestPattern {
    type Error = SyntaxError;

    fn try_from((rest, val): (Token, Expression)) -> Result<Self, Self::Error> {
        Ok(Self::new(rest, val.try_into()?))
    }
}
