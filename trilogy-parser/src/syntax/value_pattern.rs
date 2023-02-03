use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub enum ValuePattern {
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
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
    Parenthesized(Box<Pattern>),
}

#[derive(Clone, Debug)]
pub struct WildcardPattern {
    token: Token,
}

#[derive(Clone, Debug)]
pub struct NegativePattern {
    start: Token,
    pub pattern: ValuePattern,
}

#[derive(Clone, Debug)]
pub struct GluePattern {
    start: Token,
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}

#[derive(Clone, Debug)]
pub struct StructPattern {
    pub atom: AtomLiteral,
    pub pattern: ValuePattern,
    end: Token,
}

#[derive(Clone, Debug)]
pub struct TuplePattern {
    pub lhs: ValuePattern,
    pub rhs: ValuePattern,
}

#[derive(Clone, Debug)]
pub struct ArrayPattern {
    start: Token,
    pub elements: Vec<ElementPattern>,
    end: Token,
}

#[derive(Clone, Debug)]
pub struct SetPattern {
    start: Token,
    pub elements: Vec<ElementPattern>,
    end: Token,
}

#[derive(Clone, Debug)]
pub struct RecordPattern {
    start: Token,
    pub elements: Vec<RecordElementPattern>,
    end: Token,
}

#[derive(Clone, Debug)]
pub enum ElementPattern {
    Element(Pattern),
    Rest(Pattern),
}

#[derive(Clone, Debug)]
pub enum RecordElementPattern {
    Element(Pattern, Pattern),
    Rest(Pattern),
}

#[derive(Clone, Debug)]
pub struct PinnedPattern {
    start: Token,
    pub identifier: Identifier,
}

#[derive(Clone, Debug)]
pub enum Mut {
    Not,
    Mut(Token),
}

#[derive(Clone, Debug)]
pub struct BindingPattern {
    pub mutable: Option<Mut>,
    pub identifier: Identifier,
}
