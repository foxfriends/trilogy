use super::asm::Asm as _;
use crate::Value;
use std::fmt::{self, Display};
use trilogy_vm_derive::{Asm, Tags};

pub type Offset = usize;

#[derive(Debug, Tags, Asm)]
#[tags(name = OpCode, derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug), repr(u8))]
pub enum Instruction {
    Const(Value),
    Load,
    Set,
    Alloc,
    Free,
    LoadRegister(Offset),
    SetRegister(Offset),
    Copy,
    Pop,
    Swap,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    IntDivide,
    Power,
    Negate,
    Glue,
    Access,
    Assign,
    Not,
    And,
    Or,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNeg,
    LeftShift,
    RightShift,
    Cons,
    Leq,
    Lt,
    Geq,
    Gt,
    RefEq,
    ValEq,
    RefNeq,
    ValNeq,
    Call(Offset),
    Return,
    Shift(Offset),
    Reset,
    Jump(Offset),
    JumpBack(Offset),
    CondJump(Offset),
    CondJumpBack(Offset),
    Branch,
    Fizzle,
    Exit,
}

impl TryFrom<u8> for OpCode {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::Exit as u8 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(value)
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_asm(f)
    }
}
