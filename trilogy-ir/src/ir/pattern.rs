use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Pattern {
    span: Span,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub enum Value {
    Conjunction(Box<(Pattern, Pattern)>),
    Disjunction(Box<(Pattern, Pattern)>),
    Mapping(Box<(Pattern, Pattern)>),
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
    Bits(Box<BitsLiteral>),
    Boolean(Box<BooleanLiteral>),
    Unit(Box<UnitLiteral>),
    Atom(Box<AtomLiteral>),
    Negative(Box<Pattern>),
    Glue(Box<GluePattern>),
    Struct(Box<StructPattern>),
    Tuple(Box<(Pattern, Pattern)>),
    Array(Box<ArrayPattern>),
    Set(Box<SetPattern>),
    Record(Box<RecordPattern>),
    Pinned(Box<Identifier>),
    Binding(Box<Identifier>),
    Wildcard,
}
