use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Handler {
    Given(Box<GivenHandler>),
    When(Box<WhenHandler>),
    Else(Box<ElseHandler>),
}

impl Handler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        match parser.peek().token_type {
            KwGiven => Ok(Self::Given(Box::new(GivenHandler::parse(parser)?))),
            KwWhen => Ok(Self::When(Box::new(WhenHandler::parse(parser)?))),
            KwElse => Ok(Self::Else(Box::new(ElseHandler::parse(parser)?))),
            _ => unreachable!("Caller should have checked the first token"),
        }
    }
}
