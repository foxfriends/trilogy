use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ParameterList {
    pub open_paren: Token,
    pub parameters: Vec<Pattern>,
    pub close_paren: Token,
}

impl Spanned for ParameterList {
    fn span(&self) -> source_span::Span {
        self.open_paren.span.union(self.close_paren.span)
    }
}

impl ParameterList {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let mut parameters = vec![];
        let open_paren = parser
            .expect(TokenType::OParen)
            .map_err(|token| parser.expected(token, "expected `(` to begin parameter list"))?;
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
            open_paren,
            parameters,
            close_paren,
        })
    }
}
