use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::{
    Token,
    TokenType::{self, *},
};

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
    Wildcard(Box<Token>),
    Negative(Box<NegativePattern>),
    Glue(Box<GluePattern>),
    Typeof(Box<TypeofPattern>),
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
    None,
    Disjunction,
    Conjunction,
    Cons,
    Glue,
    Unary,
}

impl Pattern {
    pub(crate) const PREFIX: [TokenType; 17] = [
        Numeric, String, Bits, KwTrue, KwFalse, Atom, Character, KwUnit, OParen, OpMinus, OpCaret,
        OBrack, OBrackPipe, OBracePipe, Discard, Identifier, KwMut,
    ];

    pub(crate) fn parse_follow(
        parser: &mut Parser,
        precedence: Precedence,
        lhs: Pattern,
    ) -> SyntaxResult<Result<Self, Self>> {
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
            KwTypeof => Ok(Self::Typeof(Box::new(TypeofPattern::parse(parser)?))),
            OpCaret => Ok(Self::Pinned(Box::new(PinnedPattern::parse(parser)?))),
            OBrack => Ok(Self::Array(Box::new(ArrayPattern::parse(parser)?))),
            OBrackPipe => Ok(Self::Set(Box::new(SetPattern::parse(parser)?))),
            OBracePipe => Ok(Self::Record(Box::new(RecordPattern::parse(parser)?))),
            Discard => Ok(Self::Wildcard(Box::new(parser.expect(Discard).unwrap()))),
            KwMut | Identifier => Ok(Self::Binding(Box::new(BindingPattern::parse(parser)?))),
            _ => {
                let error = SyntaxError::new(token.span, "unexpected token in pattern");
                parser.error(error.clone());
                Err(error)
            }
        }
    }

    pub(crate) fn parse_suffix(
        parser: &mut Parser,
        precedence: Precedence,
        mut lhs: Pattern,
    ) -> SyntaxResult<Self> {
        loop {
            match Self::parse_follow(parser, precedence, lhs)? {
                Ok(updated) => lhs = updated,
                Err(lhs) => return Ok(lhs),
            }
        }
    }

    pub(crate) fn parse_precedence(
        parser: &mut Parser,
        precedence: Precedence,
    ) -> SyntaxResult<Self> {
        let lhs = Self::parse_prefix(parser)?;
        Self::parse_suffix(parser, precedence, lhs)
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        Self::parse_precedence(parser, Precedence::None)
    }
}

impl TryFrom<Expression> for Pattern {
    type Error = SyntaxError;

    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        match value {
            Expression::Reference(identifier) => Ok(Pattern::Binding(Box::new(BindingPattern {
                identifier: *identifier,
                mutable: MutModifier::Not,
            }))),
            Expression::Atom(val) => Ok(Pattern::Atom(val)),
            Expression::Number(val) => Ok(Pattern::Number(val)),
            Expression::Boolean(val) => Ok(Pattern::Boolean(val)),
            Expression::Unit(val) => Ok(Pattern::Unit(val)),
            Expression::Bits(val) => Ok(Pattern::Bits(val)),
            Expression::String(val) => Ok(Pattern::String(val)),
            Expression::Character(val) => Ok(Pattern::Character(val)),
            Expression::Unary(op) if matches!(op.operator, UnaryOperator::Negate(..)) => {
                Ok(Pattern::Negative(Box::new(NegativePattern::try_from(*op)?)))
            }
            Expression::Binary(op) if matches!(op.operator, BinaryOperator::Glue(..)) => {
                Ok(Pattern::Glue(Box::new(GluePattern::try_from(*op)?)))
            }
            Expression::Binary(op) if matches!(op.operator, BinaryOperator::Cons(..)) => {
                Ok(Pattern::Tuple(Box::new(TuplePattern::try_from(*op)?)))
            }
            Expression::Struct(inner) => {
                Ok(Pattern::Struct(Box::new(StructPattern::try_from(*inner)?)))
            }
            Expression::Array(array) => {
                Ok(Pattern::Array(Box::new(ArrayPattern::try_from(*array)?)))
            }
            Expression::Set(set) => Ok(Pattern::Set(Box::new(SetPattern::try_from(*set)?))),
            Expression::Record(record) => {
                Ok(Pattern::Record(Box::new(RecordPattern::try_from(*record)?)))
            }
            Expression::Parenthesized(paren) => Ok(Pattern::Parenthesized(Box::new(
                ParenthesizedPattern::try_from(*paren)?,
            ))),
            _ => Err(SyntaxError::new(
                value.span(),
                "expression is not valid in pattern context",
            )),
        }
    }
}
