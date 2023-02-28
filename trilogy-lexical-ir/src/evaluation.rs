use super::*;
use bitvec::vec::BitVec;
use num::{rational::BigRational, Complex};
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Evaluation {
    span: Span,
    operations: Vec<Operation>,
}

#[derive(Clone, Debug)]
enum Operation {
    Reference(Reference),
    Number(Box<Complex<BigRational>>),
    Bits(BitVec),
    String(String),
    Character(char),
    Atom(String),
    Add,
    Subtract,
    Multiply,
    Divide,
    IntDivide,
    Remainder,
    Power,
    And,
    Or,
    Not,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    Invert,
    LeftShift,
    RightShift,
    Glue,
    Cons,
    Construct,
    Push,
    Access,
    Compose,
    Apply,
    Call(usize),
}
