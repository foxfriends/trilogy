use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Query {
    Disjunction(Box<QueryDisjunction>),
    Conjunction(Box<QueryConjunction>),
    Implication(Box<QueryImplication>),
    Direct(Box<DirectUnification>),
    Element(Box<ElementUnification>),
    Parenthesized(Box<ParenthesizedQuery>),
    True(Box<Token>),
    False(Box<Token>),
    Is(Box<BooleanQuery>),
    Not(Box<NotQuery>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum Precedence {
    Primary,
    Conjunction,
    Implication,
    Disjunction,
    Not,
    None,
}

impl Query {
    fn parse_follow(
        parser: &mut Parser,
        precedence: Precedence,
        lhs: Query,
    ) -> SyntaxResult<Result<Self, Self>> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            KwAnd if precedence < Precedence::Conjunction => Ok(Ok(Self::Conjunction(Box::new(
                QueryConjunction::parse(parser, lhs)?,
            )))),
            KwOr if precedence < Precedence::Disjunction => Ok(Ok(Self::Disjunction(Box::new(
                QueryDisjunction::parse(parser, lhs)?,
            )))),
            _ => Ok(Err(lhs)),
        }
    }

    fn parse_prefix(parser: &mut Parser) -> SyntaxResult<Self> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            KwIf => Ok(Self::Implication(Box::new(QueryImplication::parse(
                parser,
            )?))),
            KwTrue => Ok(Self::True(Box::new(parser.expect(KwTrue).unwrap()))),
            KwFalse => Ok(Self::False(Box::new(parser.expect(KwFalse).unwrap()))),
            KwIs => Ok(Self::Is(Box::new(BooleanQuery::parse(parser)?))),
            OParen => Ok(Self::Parenthesized(Box::new(ParenthesizedQuery::parse(
                parser,
            )?))),
            KwNot => Ok(Self::Not(Box::new(NotQuery::parse(parser)?))),
            _ => {
                // Patterns could look like a lot of things, we'll just let the
                // pattern parser handle the errors
                let pattern = Pattern::parse(parser)?;
                let token = parser.peek();
                match token.token_type {
                    OpEq => Ok(Self::Direct(Box::new(DirectUnification::parse(
                        parser, pattern,
                    )?))),
                    KwIn => Ok(Self::Element(Box::new(ElementUnification::parse(
                        parser, pattern,
                    )?))),
                    _ => Err(SyntaxError::new(token.span, "unexpected token in query")),
                }
            }
        }
    }

    pub(crate) fn parse_precedence(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Self> {
        let mut expr = Self::parse_prefix(parser)?;
        loop {
            match Self::parse_follow(parser, precedence, expr)? {
                Ok(updated) => expr = updated,
                Err(expr) => return Ok(expr),
            }
        }
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        Self::parse_precedence(parser, Precedence::Primary)
    }
}
