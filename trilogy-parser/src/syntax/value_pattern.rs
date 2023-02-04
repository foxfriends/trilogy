use super::*;

#[derive(Clone, Debug)]
pub enum ValuePattern {
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
