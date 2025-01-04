use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ParenthesizedQuery {
    pub open_paren: Token,
    pub query: Query,
    pub close_paren: Token,
}

impl Spanned for ParenthesizedQuery {
    fn span(&self) -> Span {
        self.open_paren.span.union(self.close_paren.span())
    }
}

impl ParenthesizedQuery {
    pub(crate) fn parse_or_pattern(parser: &mut Parser) -> SyntaxResult<Result<Self, Pattern>> {
        let open_paren = parser
            .expect(OParen)
            .map_err(|token| parser.expected(token, "expected `(`"))?;
        match Query::parse_or_pattern_parenthesized(parser)? {
            Ok(query) => {
                let close_paren = parser
                    .expect(CParen)
                    .map_err(|token| parser.expected(token, "expected `)`"))?;
                Ok(Ok(Self {
                    open_paren,
                    query,
                    close_paren,
                }))
            }
            Err(pattern) => {
                let parenthesized = Pattern::Parenthesized(Box::new(ParenthesizedPattern::finish(
                    parser, open_paren, pattern,
                )?));
                let pattern =
                    Pattern::parse_suffix(parser, pattern::Precedence::None, parenthesized)?;
                Ok(Err(pattern))
            }
        }
    }
}
