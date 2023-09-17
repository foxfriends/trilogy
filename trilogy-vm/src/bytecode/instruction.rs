use crate::Value;

/// Integer type used as the single parameter to some instructions.
pub type Offset = u32;

/// An instruction for the Trilogy VM.
///
/// In bytecode form, an instruction is represented as a single-byte [`OpCode`][].
/// Some op-codes are followed by single integer parameter, whose interpretation
/// is different depending on the specific instruction.
#[rustfmt::skip]
#[derive(Debug, trilogy_vm_derive::OpCode)]
#[opcode(derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug), repr(u8))]
pub enum Instruction {
    // Stack
    Const(Value),
    Copy,
    Clone,
    Pop,
    Swap,
    TypeOf,

    // Heap (why?)
    #[opcode(name = "LOAD")] Load,
    #[opcode(name = "SET")] Set,
    #[opcode(name = "INIT")] Init,
    #[opcode(name = "UNSET")] Unset,
    Alloc,
    Free,

    // Variables
    #[opcode(name = "VAR")] Variable,
    #[opcode(name = "LOADL")] LoadLocal(Offset),
    #[opcode(name = "SETL")] SetLocal(Offset),
    #[opcode(name = "INITL")] InitLocal(Offset),
    #[opcode(name = "UNSETL")] UnsetLocal(Offset),
    #[opcode(name = "LOADR")] LoadRegister(Offset),
    #[opcode(name = "SETR")] SetRegister(Offset),

    // Numbers
    Add,
    #[opcode(name = "SUB")] Subtract,
    #[opcode(name = "MUL")] Multiply,
    #[opcode(name = "DIV")] Divide,
    #[opcode(name = "REM")] Remainder,
    #[opcode(name = "INTDIV")] IntDivide,
    #[opcode(name = "POW")] Power,
    #[opcode(name = "NEG")] Negate,

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
    #[opcode(name = "BITAND")] BitwiseAnd,
    #[opcode(name = "BITOR")] BitwiseOr,
    #[opcode(name = "BITXOR")] BitwiseXor,
    #[opcode(name = "BITNEG")] BitwiseNeg,
    #[opcode(name = "BITSHIFTL")] LeftShift,
    #[opcode(name = "BITSHIFTR")] RightShift,

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
    #[opcode(name = "JUMPF")] CondJump(Offset),
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
