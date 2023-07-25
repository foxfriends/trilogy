use super::asm::Asm as _;
use crate::Value;
use std::fmt::{self, Display};
use trilogy_vm_derive::{Asm, Tags};

pub type Offset = usize;

#[rustfmt::skip]
#[derive(Debug, Tags, Asm)]
#[tags(name = OpCode, derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug), repr(u8))]
pub enum Instruction {
    Const(Value),
    #[asm(name = "LOAD")] Load,
    #[asm(name = "SET")] Set,
    Alloc,
    Free,
    #[asm(name = "LOADL")] LoadLocal(Offset),
    #[asm(name = "SETL")] SetLocal(Offset),
    #[asm(name = "LOADR")] LoadRegister(Offset),
    #[asm(name = "SETR")] SetRegister(Offset),
    Copy,
    Clone,
    Pop,
    Swap,
    TypeOf,
    Add,
    #[asm(name = "SUB")] Subtract,
    #[asm(name = "MUL")] Multiply,
    #[asm(name = "DIV")] Divide,
    #[asm(name = "REM")] Remainder,
    #[asm(name = "INTDIV")] IntDivide,
    #[asm(name = "POW")] Power,
    #[asm(name = "NEG")] Negate,
    Glue,
    Access,
    Assign,
    Insert,
    Delete,
    Entries,
    Length,
    Not,
    And,
    Or,
    #[asm(name = "BITAND")] BitwiseAnd,
    #[asm(name = "BITOR")] BitwiseOr,
    #[asm(name = "BITXOR")] BitwiseXor,
    #[asm(name = "BITNEG")] BitwiseNeg,
    #[asm(name = "LSHIFT")] LeftShift,
    #[asm(name = "RSHIFT")] RightShift,
    Cons,
    Uncons,
    First,
    Second,
    Construct,
    Destruct,
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
    #[asm(name = "RJUMP")] JumpBack(Offset),
    #[asm(name = "JUMPF")] CondJump(Offset),
    #[asm(name = "RJUMPF")] CondJumpBack(Offset),
    Branch,
    Fizzle,
    Exit,
}

impl Instruction {
    pub fn size(&self) -> usize {
        match self {
            Self::Const(..) => 5,
            Self::LoadLocal(..) => 5,
            Self::SetLocal(..) => 5,
            Self::LoadRegister(..) => 5,
            Self::SetRegister(..) => 5,
            Self::Call(..) => 5,
            Self::Shift(..) => 5,
            Self::Jump(..) => 5,
            Self::JumpBack(..) => 5,
            Self::CondJump(..) => 5,
            Self::CondJumpBack(..) => 5,
            _ => 1,
        }
    }
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
