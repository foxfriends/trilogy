use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Lookup {
    pub path: Expression,
    pub open_paren: Token,
    pub patterns: Vec<Pattern>,
    pub close_paren: Token,
    span: Span,
}

impl Spanned for Lookup {
    fn span(&self) -> Span {
        self.span
    }
}

impl Lookup {
    pub(crate) fn parse_rest(parser: &mut Parser, path: Expression) -> SyntaxResult<Self> {
        let open_paren = parser
            .expect(TokenType::OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        let mut patterns = vec![];
        let close_paren = loop {
            if let Ok(close_paren) = parser.expect(TokenType::CParen) {
                break close_paren;
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
            span: path.span().union(close_paren.span),
            path,
            open_paren,
            patterns,
            close_paren,
        })
    }
}
