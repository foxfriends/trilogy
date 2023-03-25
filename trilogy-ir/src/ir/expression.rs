use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Expression {
    span: Span,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub enum Value {
    Builtin(Box<Builtin>),
    Pack(Box<Pack>),
    Block(Vec<Expression>),
    Mapping(Box<(Expression, Expression)>),
    Number(Box<NumberLiteral>),
    Character(Box<CharacterLiteral>),
    String(Box<StringLiteral>),
    Bits(Box<BitsLiteral>),
    Boolean(Box<BooleanLiteral>),
    Unit(Box<UnitLiteral>),
    Atom(Box<AtomLiteral>),
    ArrayComprehension(Box<ArrayComprehension>),
    SetComprehension(Box<SetComprehension>),
    RecordComprehension(Box<RecordComprehension>),
    IteratorComprehension(Box<IteratorComprehension>),
    For(Box<For>),
    While(Box<While>),
    Application(Box<Application>),
    Let(Box<Let>),
    IfElse(Box<IfElse>),
    Match(Box<Match>),
    Is(Box<Query>),
    Fn(Box<Function>),
    Do(Box<Procedure>),
    Handled(Box<Handled>),
    Module(Box<Identifier>),
    Reference(Box<Identifier>),
    Assert(Box<Assert>),
}
