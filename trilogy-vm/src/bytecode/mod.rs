#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(u8)]
pub enum Instruction {
    // Direct stack manipulation
    Const,
    Load,
    Set,
    Pop,
    // Basic operations
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
    // Control flow
    Return,
    Invert,
    Jump,
    CondJump,
    Branch,
    Fizzle,
    Exit,
}

impl TryFrom<u8> for Instruction {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value <= Self::Exit as u8 {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(value)
        }
    }
}
