use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum HandlerStrategy {
    Cancel(Token),
    Resume(Token),
    Invert(Token),
    Yield(Token),
}

impl HandlerStrategy {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect([KwCancel, KwResume, KwInvert, KwYield])
            .map_err(|token| {
                parser.expected(
                    token,
                    "expected `cancel`, `resume`, `invert`, or `yield` to determine handler type",
                )
            })?;

        match token.token_type {
            KwCancel => Ok(Self::Cancel(token)),
            KwResume => Ok(Self::Resume(token)),
            KwInvert => Ok(Self::Invert(token)),
            KwYield => Ok(Self::Yield(token)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(handler_strategy_yield: "yield" => HandlerStrategy::parse => "(HandlerStrategy::Yield _)");
    test_parse!(handler_strategy_cancel: "cancel" => HandlerStrategy::parse => "(HandlerStrategy::Cancel _)");
    test_parse!(handler_strategy_invert: "invert" => HandlerStrategy::parse => "(HandlerStrategy::Invert _)");
    test_parse!(handler_strategy_resume: "resume" => HandlerStrategy::parse => "(HandlerStrategy::Resume _)");
}
