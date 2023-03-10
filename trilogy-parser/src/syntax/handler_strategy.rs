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
