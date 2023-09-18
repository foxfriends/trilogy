use crate::{Chunk, Value};
use trilogy_vm_derive::Asm;

/// Integer type used as the single parameter to some instructions.
pub type Offset = u32;

/// An instruction for the Trilogy VM.
///
/// In bytecode form, an instruction is represented as a single-byte [`OpCode`][].
/// Some op-codes are followed by single integer parameter, whose interpretation
/// is different depending on the specific instruction.
#[rustfmt::skip]
#[derive(Debug, Asm)]
#[asm(derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug), repr(u8))]
pub enum Instruction {
    // Stack
    Const(Value),
    Copy,
    Clone,
    Pop,
    Swap,
    TypeOf,

    // Heap (why?)
    #[asm(name = "LOAD")] Load,
    #[asm(name = "SET")] Set,
    #[asm(name = "INIT")] Init,
    #[asm(name = "UNSET")] Unset,
    Alloc,
    Free,

    // Variables
    #[asm(name = "VAR")] Variable,
    #[asm(name = "LOADL")] LoadLocal(Offset),
    #[asm(name = "SETL")] SetLocal(Offset),
    #[asm(name = "INITL")] InitLocal(Offset),
    #[asm(name = "UNSETL")] UnsetLocal(Offset),
    #[asm(name = "LOADR")] LoadRegister(Offset),
    #[asm(name = "SETR")] SetRegister(Offset),

    // Numbers
    Add,
    #[asm(name = "SUB")] Subtract,
    #[asm(name = "MUL")] Multiply,
    #[asm(name = "DIV")] Divide,
    #[asm(name = "REM")] Remainder,
    #[asm(name = "INTDIV")] IntDivide,
    #[asm(name = "POW")] Power,
    #[asm(name = "NEG")] Negate,

    // Collections
    Access,
    Assign,
    Insert,
    Delete,
    Contains,
    Entries,
    Length,
    Take,
    Skip,
    Glue,

    // Booleans
    Not,
    And,
    Or,

    // Bits
    #[asm(name = "BITAND")] BitwiseAnd,
    #[asm(name = "BITOR")] BitwiseOr,
    #[asm(name = "BITXOR")] BitwiseXor,
    #[asm(name = "BITNEG")] BitwiseNeg,
    #[asm(name = "BITSHIFTL")] LeftShift,
    #[asm(name = "BITSHIFTR")] RightShift,

    // Tuples
    Cons,
    Uncons,
    First,
    Second,

    // Structs
    Construct,
    Destruct,

    // Comparison
    Leq,
    Lt,
    Geq,
    Gt,
    RefEq,
    ValEq,
    RefNeq,
    ValNeq,

    // Control Flow
    Call(Offset),
    Become(Offset),
    Return,
    Close(Offset),
    Shift(Offset),
    Reset,
    Jump(Offset),
    #[asm(name = "JUMPF")] CondJump(Offset),
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

trait FromChunk {
    fn from_chunk(chunk: &Chunk, offset: Offset) -> Self;
}

impl FromChunk for Offset {
    fn from_chunk(chunk: &Chunk, offset: Offset) -> Self {
        chunk.offset(offset)
    }
}

impl FromChunk for Value {
    fn from_chunk(chunk: &Chunk, offset: Offset) -> Self {
        chunk.constant(offset)
    }
}
