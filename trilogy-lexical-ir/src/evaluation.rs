use super::*;
use bitvec::vec::BitVec;
use num::{rational::BigRational, Complex};
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Evaluation {
    pub span: Span,
    pub value: Value,
}

#[derive(Clone, Debug)]
pub enum Value {
    DynamicResolve(String),
    Declaration(Box<Reference>),
    Reference(Box<Reference>),
    Dereference(Box<Reference>),
    Number(Box<Complex<BigRational>>),
    Bits(BitVec),
    String(String),
    Character(char),
    Atom(String),
    Wildcard,
    It,
    Yield(Box<Evaluation>),
    Resume(Box<Evaluation>),
    Cancel(Box<Evaluation>),
    Break(Box<Evaluation>),
    Continue(Box<Evaluation>),
    Return(Box<Evaluation>),
    Exit(Box<Evaluation>),
    Mapping(Box<BinaryOperation>),
    Add(Box<BinaryOperation>),
    Subtract(Box<BinaryOperation>),
    Multiply(Box<BinaryOperation>),
    Divide(Box<BinaryOperation>),
    IntDivide(Box<BinaryOperation>),
    Remainder(Box<BinaryOperation>),
    Power(Box<BinaryOperation>),
    Negate(Box<Evaluation>),
    And(Box<BinaryOperation>),
    Or(Box<BinaryOperation>),
    Not(Box<Evaluation>),
    BitwiseAnd(Box<BinaryOperation>),
    BitwiseOr(Box<BinaryOperation>),
    BitwiseXor(Box<BinaryOperation>),
    Invert(Box<Evaluation>),
    LeftShift(Box<BinaryOperation>),
    RightShift(Box<BinaryOperation>),
    Glue(Box<BinaryOperation>),
    Cons(Box<BinaryOperation>),
    Access(Box<BinaryOperation>),
    Compose(Box<BinaryOperation>),
    Apply(Box<BinaryOperation>),
    Call(Box<Call>),
    Cond(Vec<Cond>),
    Branch(Box<Direction>),
    Collect(Box<Collect>),
    Module(Box<Reference>),
    ModuleAccess(Box<BinaryOperation>),
}

#[derive(Clone, Debug)]
pub struct BinaryOperation {
    pub lhs: Evaluation,
    pub rhs: Evaluation,
}

#[derive(Clone, Debug)]
pub struct Call {
    pub func: Evaluation,
    pub args: Vec<Evaluation>,
}

#[derive(Clone, Debug)]
pub struct Cond {
    pub cond: Vec<Code>,
    pub body: Vec<Code>,
}
