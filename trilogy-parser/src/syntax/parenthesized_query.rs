use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ParenthesizedQuery {
    pub oparen: Token,
    pub query: Query,
    pub cparen: Token,
}

impl ParenthesizedQuery {
    pub(crate) fn parse_or_pattern(parser: &mut Parser) -> SyntaxResult<Result<Self, Pattern>> {
        let oparen = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        match Query::parse_or_pattern_parenthesized(parser)? {
            Ok(query) => {
                let cparen = parser
                    .expect(CParen)
                    .map_err(|token| parser.expected(token, "expected `)`"))?;
                Ok(Ok(Self {
                    oparen,
                    query,
                    cparen,
                }))
            }
            Err(pattern) => Ok(Err(Pattern::Parenthesized(Box::new(
                ParenthesizedPattern::finish(parser, oparen, pattern)?,
            )))),
        }
    }
}
