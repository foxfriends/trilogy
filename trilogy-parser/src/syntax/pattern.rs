use super::*;
use crate::Parser;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum Pattern {
    Conjunction(Box<PatternConjunction>),
    Disjunction(Box<PatternDisjunction>),
    Typed(Box<TypedPattern>),
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

impl Pattern {
    pub(crate) fn parse(_parser: &mut Parser) -> SyntaxResult<Self> {
        todo!()
    }
}
