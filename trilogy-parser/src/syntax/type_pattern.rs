use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub enum TypePattern {
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
    Boolean(Box<BooleanLiteral>),
    Unit(Box<UnitLiteral>),
    Atom(Box<AtomLiteral>),
    Tuple(Box<TupleType>),
    Struct(Box<StructType>),
    Identifier(Box<Identifier>),
}

#[derive(Clone, Debug)]
pub struct TupleType {
    pub lhs: TypePattern,
    pub rhs: TypePattern,
}

#[derive(Clone, Debug)]
pub struct StructType {
    pub atom: AtomLiteral,
    pub pattern: TypePattern,
    end: Token,
}

#[derive(Clone, Debug)]
pub struct ArrayType {
    start: Token,
    pub pattern: TypePattern,
}

#[derive(Clone, Debug)]
pub struct SetType {
    start: Token,
    pub pattern: TypePattern,
}

#[derive(Clone, Debug)]
pub struct RecordType {
    start: Token,
    pub key_pattern: TypePattern,
    pub value_pattern: TypePattern,
}
