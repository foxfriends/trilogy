use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum HandlerStrategy {
    Cancel { cancel: Token, body: Expression },
    Resume { resume: Token, body: Expression },
    Then { invert: Token, body: Block },
    Yield(Token),
}

impl HandlerStrategy {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect([KwCancel, KwResume, KwThen, KwYield])
            .map_err(|token| {
                parser.expected(
                    token,
                    "expected `cancel`, `resume`, `then`, or `yield` to determine handler type",
                )
            })?;

        match token.token_type {
            KwCancel => Ok(Self::Cancel {
                cancel: token,
                body: Expression::parse(parser)?,
            }),
            KwResume => Ok(Self::Resume {
                resume: token,
                body: Expression::parse(parser)?,
            }),
            KwThen => Ok(Self::Then {
                invert: token,
                body: Block::parse(parser)?,
            }),
            KwYield => Ok(Self::Yield(token)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(handler_strategy_yield: "yield" => HandlerStrategy::parse => "(HandlerStrategy::Yield _)");
    test_parse!(handler_strategy_cancel: "cancel 3" => HandlerStrategy::parse => "(HandlerStrategy::Cancel _ _)");
    test_parse!(handler_strategy_invert: "then {}" => HandlerStrategy::parse => "(HandlerStrategy::Then _ _)");
    test_parse!(handler_strategy_resume: "resume 4" => HandlerStrategy::parse => "(HandlerStrategy::Resume _ _)");
}
