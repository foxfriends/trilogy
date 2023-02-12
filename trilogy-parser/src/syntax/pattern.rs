use super::*;
use crate::Parser;
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Pattern {
    Conjunction(Box<PatternConjunction>),
    Disjunction(Box<PatternDisjunction>),
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
    Bits(Box<BitsLiteral>),
    Boolean(Box<BooleanLiteral>),
    Unit(Box<UnitLiteral>),
    Atom(Box<AtomLiteral>),
    Wildcard(Box<WildcardPattern>),
    Negative(Box<NegativePattern>),
    Glue(Box<GluePattern>),
    Struct(Box<StructPattern>),
    Tuple(Box<TuplePattern>),
    Array(Box<ArrayPattern>),
    Set(Box<SetPattern>),
    Record(Box<RecordPattern>),
    Pinned(Box<PinnedPattern>),
    Binding(Box<BindingPattern>),
    Parenthesized(Box<ParenthesizedPattern>),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum Precedence {
    Primary,
    Unary,
    Glue,
    Cons,
    Conjunction,
    Disjunction,
    None,
}

impl Pattern {
    fn parse_follow(
        parser: &mut Parser,
        precedence: Precedence,
        lhs: Pattern,
    ) -> SyntaxResult<Result<Self, Self>> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            KwAnd if precedence < Precedence::Conjunction => Ok(Ok(Self::Conjunction(Box::new(
                PatternConjunction::parse(parser, lhs)?,
            )))),
            KwOr if precedence < Precedence::Disjunction => Ok(Ok(Self::Disjunction(Box::new(
                PatternDisjunction::parse(parser, lhs)?,
            )))),
            OpGlue if precedence < Precedence::Glue => {
                Ok(Ok(Self::Glue(Box::new(GluePattern::parse(parser, lhs)?))))
            }
            OpColon if precedence <= Precedence::Cons => {
                Ok(Ok(Self::Tuple(Box::new(TuplePattern::parse(parser, lhs)?))))
            }
            _ => Ok(Err(lhs)),
        }
    }

    fn parse_prefix(parser: &mut Parser) -> SyntaxResult<Self> {
        use TokenType::*;
        let token = parser.peek();
        match token.token_type {
            Numeric => Ok(Self::Number(Box::new(NumberLiteral::parse(parser)?))),
            String => Ok(Self::String(Box::new(StringLiteral::parse(parser)?))),
            Bits => Ok(Self::Bits(Box::new(BitsLiteral::parse(parser)?))),
            KwTrue | KwFalse => Ok(Self::Boolean(Box::new(BooleanLiteral::parse(parser)?))),
            Atom => {
                let atom = AtomLiteral::parse(parser)?;
                if parser.check(OParen).is_ok() {
                    Ok(Self::Struct(Box::new(StructPattern::parse(parser, atom)?)))
                } else {
                    Ok(Self::Atom(Box::new(atom)))
                }
            }
            Character => Ok(Self::Character(Box::new(CharacterLiteral::parse(parser)?))),
            KwUnit => Ok(Self::Unit(Box::new(UnitLiteral::parse(parser)?))),
            OParen => Ok(Self::Parenthesized(Box::new(ParenthesizedPattern::parse(
                parser,
            )?))),
            OpMinus => Ok(Self::Negative(Box::new(NegativePattern::parse(parser)?))),
            OpCaret => Ok(Self::Pinned(Box::new(PinnedPattern::parse(parser)?))),
            _ => Err(SyntaxError::new(token.span, "unexpected token in pattern")),
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
