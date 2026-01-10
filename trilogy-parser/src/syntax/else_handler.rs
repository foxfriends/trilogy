use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// A fallback `else` handler for a handled `when` statement or expression.
#[derive(Clone, Debug)]
pub struct ElseHandler {
    pub r#else: Token,
    pub strategy: HandlerStrategy,
    pub span: Span,
}

impl ElseHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#else = parser.expect(TokenType::KwElse).unwrap();
        let strategy = HandlerStrategy::parse(parser)?;

        Ok(Self {
            span: r#else.span.union(strategy.span()),
            r#else,
            strategy,
        })
    }

    pub fn else_token(&self) -> &Token {
        &self.r#else
    }
}

impl Spanned for ElseHandler {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(elsehandler_yield: "else yield" => ElseHandler::parse => ElseHandler { .. });
    test_parse!(elsehandler_resume: "else resume 3" => ElseHandler::parse => ElseHandler { .. });
    test_parse!(elsehandler_cancel: "else cancel 3" => ElseHandler::parse => ElseHandler { .. });
}
