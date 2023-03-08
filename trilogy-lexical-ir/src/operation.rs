use super::*;
use bitvec::vec::BitVec;
use num::{rational::BigRational, Complex};

#[derive(Clone, Debug)]
pub enum Operation {
    Reference(Box<Reference>),
    Dereference(Box<Reference>),
    Number(Box<Complex<BigRational>>),
    Bits(BitVec),
    String(String),
    Character(char),
    Atom(String),
    Wildcard,
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
    Collect(Box<Collect>),
}

#[derive(Clone, Debug)]
pub struct BinaryOperation {
    lhs: Evaluation,
    rhs: Evaluation,
}

#[derive(Clone, Debug)]
pub struct Call {
    func: Evaluation,
    args: Vec<Evaluation>,
}
