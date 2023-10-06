use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Handler {
    When(Box<WhenHandler>),
    Else(Box<ElseHandler>),
}

impl Handler {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        match parser.peek().token_type {
            KwWhen => Ok(Self::When(Box::new(WhenHandler::parse(parser)?))),
            KwElse => Ok(Self::Else(Box::new(ElseHandler::parse(parser)?))),
            _ => unreachable!("Caller should have checked the first token"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(handler_when: "when 'NAN resume 5" => Handler::parse => "(Handler::When (WhenHandler _ _ _ _))");
    test_parse!(handler_else: "else n resume 5" => Handler::parse => "(Handler::Else (ElseHandler _ _ _))");
}
