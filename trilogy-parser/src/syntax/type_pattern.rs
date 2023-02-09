use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
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

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TupleType {
    pub lhs: TypePattern,
    pub rhs: TypePattern,
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct StructType {
    pub atom: AtomLiteral,
    pub pattern: TypePattern,
    end: Token,
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ArrayType {
    start: Token,
    pub pattern: TypePattern,
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct SetType {
    start: Token,
    pub pattern: TypePattern,
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct RecordType {
    start: Token,
    pub key_pattern: TypePattern,
    pub value_pattern: TypePattern,
}
