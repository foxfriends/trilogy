use super::{Array, Atom, Record, ReferentialEq, Set, Struct, StructuralEq, Tuple};
use bitvec::vec::BitVec;
use num::{BigRational, Complex};

pub type Number = Complex<BigRational>;
pub type Bits = BitVec;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum Value {
    Unit,
    Bool(bool),
    Char(char),
    String(String),
    Number(Box<Number>),
    Bits(Bits),
    Atom(Atom),
    Struct(Box<Struct>),
    Tuple(Box<Tuple>),
    Array(Box<Array>),
    Set(Box<Set>),
    Record(Box<Record>),

    // TODO: these will require some thought
    Iterator,
    Function,
    Procedure,
    Rule,
    Module,
    Continuation,
}

impl ReferentialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(lhs), Self::Array(rhs)) => ReferentialEq::eq(&**lhs, &**rhs),
            (Self::Record(lhs), Self::Record(rhs)) => ReferentialEq::eq(&**lhs, &**rhs),
            (Self::Set(lhs), Self::Set(rhs)) => ReferentialEq::eq(&**lhs, &**rhs),
            _ => self == other,
        }
    }
}

impl StructuralEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Array(lhs), Self::Array(rhs)) => StructuralEq::eq(&**lhs, &**rhs),
            (Self::Record(lhs), Self::Record(rhs)) => StructuralEq::eq(&**lhs, &**rhs),
            (Self::Set(lhs), Self::Set(rhs)) => StructuralEq::eq(&**lhs, &**rhs),
            _ => self == other,
        }
    }
}
