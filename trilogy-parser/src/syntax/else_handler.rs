use super::{Identifier, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{
    Token,
    TokenType::{self, *},
};

/// A fallback `else` handler for a handled `when` statement or expression.
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ElseHandler {
    pub r#else: Token,
    pub identifier: Option<Identifier>,
    pub strategy: HandlerStrategy,
}

impl ElseHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#else = parser
            .expect(KwElse)
            .expect("Caller should have found this");

        let identifier = if parser.check(TokenType::Identifier).is_ok() {
            Some(Identifier::parse(parser).unwrap())
        } else {
            None
        };

        let strategy = HandlerStrategy::parse(parser)?;

        Ok(Self {
            r#else,
            identifier,
            strategy,
        })
    }

    pub fn else_token(&self) -> &Token {
        &self.r#else
    }
}

impl Spanned for ElseHandler {
    fn span(&self) -> Span {
        self.r#else.span.union(self.strategy.span())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(elsehandler_yield: "else yield" => ElseHandler::parse => "(ElseHandler _ () _)");
    test_parse!(elsehandler_resume_without_id: "else resume 3" => ElseHandler::parse => "(ElseHandler _ () _)");
    test_parse!(elsehandler_resume_with_id: "else x resume x" => ElseHandler::parse => "(ElseHandler _ (Identifier) _)");
    test_parse!(elsehandler_cancel_without_id: "else cancel 3" => ElseHandler::parse => "(ElseHandler _ () _)");
    test_parse!(elsehandler_cancel_with_id: "else x cancel x" => ElseHandler::parse => "(ElseHandler _ (Identifier) _)");
}
