use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct ParameterList {
    pub open_paren: Token,
    pub parameters: Vec<Pattern>,
    pub close_paren: Token,
    pub span: Span,
}

impl Spanned for ParameterList {
    fn span(&self) -> Span {
        self.span
    }
}

impl ParameterList {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let open_paren = parser
            .expect(TokenType::OParen)
            .map_err(|token| parser.expected(token, "expected `(` to begin parameter list"))?;
        Self::parse_opened(parser, open_paren)
    }

    pub(crate) fn parse_opened(parser: &mut Parser, open_paren: Token) -> SyntaxResult<Self> {
        let mut parameters = vec![];
        let close_paren = loop {
            if let Ok(paren) = parser.expect(TokenType::CParen) {
                break paren;
            }
            parameters.push(Pattern::parse(parser)?);
            if parser.expect(TokenType::OpComma).is_ok() {
                continue;
            }
            let close_paren = parser
                .expect(TokenType::CParen)
                .map_err(|token| parser.expected(token, "expected `)` to end parameter list"))?;
            break close_paren;
        };
        Ok(Self {
            span: open_paren.span.union(close_paren.span),
            open_paren,
            parameters,
            close_paren,
        })
    }
}
