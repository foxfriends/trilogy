use super::*;
use bitvec::vec::BitVec;
use num::{rational::BigRational, Complex};
use source_span::Span;

#[derive(Clone, Debug)]
pub enum Value {
    Declaration(Box<Reference>),
    Reference(Box<Reference>),
    Dereference(Box<Reference>),
    Parameter(usize),
    Number(Box<Complex<BigRational>>),
    Bits(BitVec),
    Boolean(bool),
    String(String),
    Character(char),
    Atom(String),
    Unit,
    Wildcard,
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
    StaticResolve(String),
    ApplyModule(Box<BinaryOperation>),
    AccessModule(Box<BinaryOperation>),
}

macro_rules! binary {
    ($name:ident, $variant:ident) => {
        pub fn $name(lhs: Evaluation, rhs: Evaluation) -> Self {
            Self::$variant(Box::new(BinaryOperation::new(lhs, rhs)))
        }
    };
}

macro_rules! unary {
    ($name:ident, $variant:ident) => {
        unary!($name, $variant, Evaluation);
    };
    ($name:ident, $variant:ident, $t:ty) => {
        pub fn $name(value: $t) -> Self {
            Self::$variant(Box::new(value))
        }
    };
}

impl Value {
    unary!(not, Not);
    unary!(negate, Negate);
    unary!(invert, Invert);
    unary!(r#yield, Yield);
    unary!(resume, Resume);
    unary!(cancel, Cancel);
    unary!(r#break, Break);
    unary!(r#continue, Continue);
    unary!(r#return, Return);
    unary!(exit, Exit);

    binary!(mapping, Mapping);
    binary!(add, Add);
    binary!(subtract, Subtract);
    binary!(multiply, Multiply);
    binary!(divide, Divide);
    binary!(int_divide, IntDivide);
    binary!(remainder, Remainder);
    binary!(power, Power);
    binary!(and, And);
    binary!(or, Or);
    binary!(bitwiseand, BitwiseAnd);
    binary!(bitwiseor, BitwiseOr);
    binary!(bitwisexor, BitwiseXor);
    binary!(leftshift, LeftShift);
    binary!(rightshift, RightShift);
    binary!(glue, Glue);
    binary!(cons, Cons);
    binary!(access, Access);
    binary!(compose, Compose);
    binary!(apply, Apply);
    binary!(apply_module, ApplyModule);
    binary!(access_module, AccessModule);

    unary!(call, Call, Call);
    unary!(branch, Branch, Direction);
    unary!(collect, Collect, Collect);

    unary!(declaration, Declaration, Reference);
    unary!(dereference, Dereference, Reference);
    unary!(reference, Reference, Reference);

    pub fn static_resolve(name: impl Into<String>) -> Self {
        Self::StaticResolve(name.into())
    }

    pub fn atom(string: String) -> Self {
        Self::Atom(string)
    }

    pub fn at(self, span: Span) -> Evaluation {
        Evaluation { span, value: self }
    }
}

impl From<Complex<BigRational>> for Value {
    fn from(value: Complex<BigRational>) -> Self {
        Self::Number(Box::new(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<BitVec> for Value {
    fn from(value: BitVec) -> Self {
        Self::Bits(value)
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Self::Character(value)
    }
}
