use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum HandlerStrategy {
    Cancel { cancel: Token, body: Expression },
    Resume { resume: Token, body: Expression },
    Yield(Token),
    Bare(FollowingExpression),
}

impl HandlerStrategy {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect([KwCancel, KwResume, KwThen, KwYield, OBrace])
            .map_err(|token| {
                parser.expected(
                    token,
                    "expected `cancel`, `resume`, `then`, `yield`, or a block in handler",
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
            KwYield => Ok(Self::Yield(token)),
            KwThen | OBrace => Ok(Self::Bare(FollowingExpression::parse(parser)?)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(handler_strategy_yield: "yield" => HandlerStrategy::parse => "(HandlerStrategy::Yield _)");
    test_parse!(handler_strategy_cancel: "cancel 3" => HandlerStrategy::parse => "(HandlerStrategy::Cancel _ _)");
    test_parse!(handler_strategy_then: "then cancel resume 5" => HandlerStrategy::parse => "(HandlerStrategy::Bare _)");
    test_parse!(handler_strategy_block: "{ cancel resume 5 }" => HandlerStrategy::parse => "(HandlerStrategy::Bare _)");
    test_parse!(handler_strategy_resume: "resume 4" => HandlerStrategy::parse => "(HandlerStrategy::Resume _ _)");
}
