use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RestPattern {
    pub rest: Token,
    pub pattern: Option<Pattern>,
}

impl Spanned for RestPattern {
    fn span(&self) -> Span {
        match &self.pattern {
            Some(pat) => self.rest.span.union(pat.span()),
            None => self.rest.span,
        }
    }
}

impl RestPattern {
    pub(crate) fn new(rest: Token, pattern: Pattern) -> Self {
        Self {
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
                rest,
                pattern: None,
            });
        }
        let pattern = Pattern::parse(parser)?;
        Ok(Self {
            rest,
            pattern: Some(pattern),
        })
    }
}

impl TryFrom<(Token, Expression)> for RestPattern {
    type Error = SyntaxError;

    fn try_from((rest, val): (Token, Expression)) -> Result<Self, Self::Error> {
        Ok(Self {
            rest,
            pattern: Some(val.try_into()?),
        })
    }
}
