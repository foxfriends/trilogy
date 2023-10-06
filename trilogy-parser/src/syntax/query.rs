use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Query {
    Disjunction(Box<QueryDisjunction>),
    Conjunction(Box<QueryConjunction>),
    Implication(Box<QueryImplication>),
    Alternative(Box<QueryAlternative>),
    Direct(Box<DirectUnification>),
    Element(Box<ElementUnification>),
    Parenthesized(Box<ParenthesizedQuery>),
    Lookup(Box<Lookup>),
    Pass(Box<Token>),
    End(Box<Token>),
    Is(Box<BooleanQuery>),
    Not(Box<NotQuery>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum Precedence {
    None,
    Not,
    Disjunction,
    Implication,
    Conjunction,
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
            KwElse if precedence < Precedence::Disjunction => Ok(Ok(Self::Alternative(Box::new(
                QueryAlternative::parse(parser, lhs)?,
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
            KwPass => Ok(Self::Pass(Box::new(parser.expect(KwPass).unwrap()))),
            KwEnd => Ok(Self::End(Box::new(parser.expect(KwEnd).unwrap()))),
            KwIs => Ok(Self::Is(Box::new(BooleanQuery::parse(parser)?))),
            // Since a query may start with a pattern, and a pattern might be parenthesized,
            // it's a bit trickier. We'll try to parse a query first, and if it fails, then
            // it's a query that starts with a pattern. Once the pattern is parsed, we can
            // check if the parentheses are ended there, or if there's an `=` or `in`, which
            // would indicate that it's actually a query
            OParen => match ParenthesizedQuery::parse_or_pattern(parser)? {
                Ok(query) => Ok(Self::Parenthesized(Box::new(query))),
                Err(pattern) => Self::unification(parser, pattern),
            },
            KwNot => Ok(Self::Not(Box::new(NotQuery::parse(parser)?))),
            Identifier => {
                // When it's an identifier it may be a lookup (which starts with any expression),
                // or just a very long and complicated pattern. Parse it as an expression until
                // it cannot be, then assume it is a pattern. If the expression completes
                match Expression::parse_or_pattern(parser)? {
                    Ok(expr) if parser.check(OParen).is_ok() => {
                        // If the next character is `(`, then this is a lookup
                        Ok(Self::Lookup(Box::new(Lookup::parse_rest(parser, expr)?)))
                    }
                    Ok(expr) => {
                        // It was not a lookup, so let's try to convert it to a pattern
                        let pattern = expr.try_into().map_err(|err: SyntaxError| {
                            parser.error(err.clone());
                            err
                        })?;
                        Self::unification(parser, pattern)
                    }
                    Err(pattern) => {
                        // It was not an expression, so is a pattern
                        Self::unification(parser, pattern)
                    }
                }
            }
            _ => {
                // Patterns could look like a lot of things, we'll just let the
                // pattern parser handle the errors
                let pattern = Pattern::parse(parser)?;
                Self::unification(parser, pattern)
            }
        }
    }

    fn unification(parser: &mut Parser, pattern: Pattern) -> SyntaxResult<Self> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            OpEq => Ok(Self::Direct(Box::new(DirectUnification::parse(
                parser, pattern,
            )?))),
            KwIn => Ok(Self::Element(Box::new(ElementUnification::parse(
                parser, pattern,
            )?))),
            _ => {
                let error = SyntaxError::new(token.span, "unexpected token in query");
                parser.error(error.clone());
                Err(error)
            }
        }
    }

    pub(crate) fn parse_precedence(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Self> {
        let lhs = Self::parse_prefix(parser)?;
        Self::parse_precedence_followers(parser, precedence, lhs)
    }

    pub(crate) fn parse_precedence_followers(
        parser: &mut Parser,
        precedence: Precedence,
        mut lhs: Query,
    ) -> SyntaxResult<Self> {
        loop {
            match Self::parse_follow(parser, precedence, lhs)? {
                Ok(updated) => lhs = updated,
                Err(query) => return Ok(query),
            }
        }
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        Self::parse_precedence(parser, Precedence::None)
    }

    pub(crate) fn parse_or_pattern_parenthesized(
        parser: &mut Parser,
    ) -> SyntaxResult<Result<Self, Pattern>> {
        use TokenType::*;
        if parser
            .check([KwIf, KwPass, KwEnd, KwIs, KwNot, OParen])
            .is_ok()
        {
            // Definitely a query
            Self::parse_precedence(parser, Precedence::None).map(Ok)
        } else if parser.check(OParen).is_ok() {
            // Nested again!
            Ok(ParenthesizedQuery::parse_or_pattern(parser)?
                .map(|query| Self::Parenthesized(Box::new(query))))
        } else {
            // Starts with a pattern, but might be a unification
            let pattern = Pattern::parse(parser)?;
            if parser.check(CParen).is_ok() {
                // Some parentheses have ended, the caller is going to be expecting to handle
                // that.
                Ok(Err(pattern))
            } else {
                // This pattern is part of a unification, so let's finish that unification, and then
                // carry on with the rest of a query until the parentheses are finally closed.
                let unification = Self::unification(parser, pattern)?;
                Ok(Ok(Self::parse_precedence_followers(
                    parser,
                    Precedence::None,
                    unification,
                )?))
            }
        }
    }
}
