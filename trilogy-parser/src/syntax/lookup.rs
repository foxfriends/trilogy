use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Lookup {
    pub path: Expression,
    pub patterns: Vec<Pattern>,
    end: Token,
}

impl Spanned for Lookup {
    fn span(&self) -> Span {
        self.path.span().union(self.end.span)
    }
}

impl Lookup {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let path = Expression::parse(parser)?;
        parser
            .expect(TokenType::OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let mut patterns = vec![];
        let end = loop {
            if let Ok(end) = parser.expect(TokenType::CParen) {
                break end;
            }
            patterns.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            break parser.expect(TokenType::CParen).map_err(|token| {
                parser.expected(token, "expected `,` or `)` in parameter list")
            })?;
        };
        Ok(Self {
            path,
            patterns,
            end,
        })
    }
}
