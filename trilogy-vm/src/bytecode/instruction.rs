use crate::Value;
use std::fmt::{self, Display};
use trilogy_vm_derive::Tags;

pub type Offset = usize;

#[derive(Debug, Tags)]
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
        match self {
            Self::Const(value) => write!(f, "CONST {value}"),
            Self::Load => write!(f, "LOAD"),
            Self::Set => write!(f, "SET"),
            Self::Alloc => write!(f, "ALLOC"),
            Self::Free => write!(f, "FREE"),
            Self::LoadRegister(offset) => write!(f, "LOADR {offset}"),
            Self::SetRegister(offset) => write!(f, "SETR {offset}"),
            Self::Pop => write!(f, "POP"),
            Self::Swap => write!(f, "SWAP"),
            Self::Copy => write!(f, "COPY"),
            Self::Add => write!(f, "ADD"),
            Self::Subtract => write!(f, "SUB"),
            Self::Multiply => write!(f, "MUL"),
            Self::Divide => write!(f, "DIV"),
            Self::Remainder => write!(f, "REM"),
            Self::IntDivide => write!(f, "INTDIV"),
            Self::Power => write!(f, "POW"),
            Self::Negate => write!(f, "NEG"),
            Self::Glue => write!(f, "GLUE"),
            Self::Access => write!(f, "ACCESS"),
            Self::Assign => write!(f, "ASSIGN"),
            Self::Not => write!(f, "NOT"),
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
            Self::BitwiseAnd => write!(f, "BITAND"),
            Self::BitwiseOr => write!(f, "BITOR"),
            Self::BitwiseXor => write!(f, "BITXOR"),
            Self::BitwiseNeg => write!(f, "BITNEG"),
            Self::LeftShift => write!(f, "LSHIFT"),
            Self::RightShift => write!(f, "RSHIFT"),
            Self::Cons => write!(f, "CONS"),
            Self::Leq => write!(f, "LEQ"),
            Self::Lt => write!(f, "LT"),
            Self::Geq => write!(f, "GEQ"),
            Self::Gt => write!(f, "GT"),
            Self::RefEq => write!(f, "REFEQ"),
            Self::ValEq => write!(f, "VALEQ"),
            Self::RefNeq => write!(f, "REFNEQ"),
            Self::ValNeq => write!(f, "VALNEQ"),
            Self::Call(offset) => write!(f, "CALL {offset}"),
            Self::Return => write!(f, "RETURN"),
            Self::Shift(offset) => write!(f, "SHIFT {offset}"),
            Self::Reset => write!(f, "RESET"),
            Self::Jump(offset) => write!(f, "JUMP {offset}"),
            Self::JumpBack(offset) => write!(f, "RJUMP {offset}"),
            Self::CondJump(offset) => write!(f, "JUMPF {offset}"),
            Self::CondJumpBack(offset) => write!(f, "RJUMPF {offset}"),
            Self::Branch => write!(f, "BRANCH"),
            Self::Fizzle => write!(f, "FIZZLE"),
            Self::Exit => write!(f, "EXIT"),
        }
    }
}
