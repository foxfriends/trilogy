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
        let token = match parser.check([KwCancel, KwResume, KwThen, KwYield, OBrace]) {
            Ok(token) => token,
            Err(token) => {
                let token = token.clone();
                return Err(parser.expected(
                    token,
                    "expected `cancel`, `resume`, `then`, `yield`, or a block in handler",
                ));
            }
        };

        match token.token_type {
            KwCancel => Ok(Self::Cancel {
                cancel: parser.expect(KwCancel).unwrap(),
                body: Expression::parse(parser)?,
            }),
            KwResume => Ok(Self::Resume {
                resume: parser.expect(KwResume).unwrap(),
                body: Expression::parse(parser)?,
            }),
            KwYield => Ok(Self::Yield(parser.expect(KwYield).unwrap())),
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
