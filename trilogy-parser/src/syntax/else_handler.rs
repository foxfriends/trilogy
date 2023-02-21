use super::{Identifier, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{
    Token,
    TokenType::{self, *},
};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ElseHandler {
    start: Token,
    pub identifier: Option<Identifier>,
    pub strategy: HandlerStrategy,
    pub body: Option<HandlerBody>,
}

impl ElseHandler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwElse)
            .expect("Caller should have found this");

        let identifier = if parser.check(TokenType::Identifier).is_ok() {
            Some(Identifier::parse(parser).unwrap())
        } else {
            None
        };

        let strategy = HandlerStrategy::parse(parser)?;
        let body = if !matches!(strategy, HandlerStrategy::Yield(..)) {
            Some(HandlerBody::parse(parser)?)
        } else {
            None
        };

        Ok(Self {
            start,
            identifier,
            strategy,
            body,
        })
    }
}

impl Spanned for ElseHandler {
    fn span(&self) -> Span {
        match &self.body {
            None => self.start.span.union(self.strategy.span()),
            Some(body) => self.start.span.union(body.span()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(elsehandler_yield: "else yield" => ElseHandler::parse => "(ElseHandler () (HandlerStrategy::Yield _) ())");
    test_parse!(elsehandler_resume_without_id: "else resume 3" => ElseHandler::parse => "(ElseHandler () (HandlerStrategy::Resume _) (HandlerBody::Expression _))");
    test_parse!(elsehandler_resume_with_id: "else x resume x" => ElseHandler::parse => "(ElseHandler (Identifier) (HandlerStrategy::Resume _) (HandlerBody::Expression _))");
    test_parse!(elsehandler_cancel_without_id: "else cancel 3" => ElseHandler::parse => "(ElseHandler () (HandlerStrategy::Cancel _) (HandlerBody::Expression _))");
    test_parse!(elsehandler_cancel_with_id: "else x cancel x" => ElseHandler::parse => "(ElseHandler (Identifier) (HandlerStrategy::Cancel _) (HandlerBody::Expression _))");
    test_parse!(elsehandler_invert_without_id: "else invert 3" => ElseHandler::parse => "(ElseHandler () (HandlerStrategy::Invert _) (HandlerBody::Expression _))");
    test_parse!(elsehandler_invert_with_id: "else x invert x" => ElseHandler::parse => "(ElseHandler (Identifier) (HandlerStrategy::Invert _) (HandlerBody::Expression _))");
    test_parse_error!(elsehandler_invalid_body: "else invert { exit }" => ElseHandler::parse);
    test_parse_error!(elsehandler_not_identifier: "else (x) invert { exit }" => ElseHandler::parse);
}
