use crate::program::ProgramContext;
use trilogy_vm::Instruction;

pub const ADD: &str = "core::add";
pub const SUB: &str = "core::sub";
pub const MUL: &str = "core::mul";
pub const DIV: &str = "core::div";
pub const INTDIV: &str = "core::intdiv";
pub const REM: &str = "core::rem";
pub const POW: &str = "core::pow";
pub const NEGATE: &str = "core::neg";

pub const GLUE: &str = "core::glue";

pub const AND: &str = "core::and";
pub const OR: &str = "core::or";
pub const NOT: &str = "core::not";

pub const BITAND: &str = "core::bitand";
pub const BITOR: &str = "core::bitor";
pub const BITXOR: &str = "core::bitxor";
pub const BITNEG: &str = "core::bitneg";
pub const LSHIFT: &str = "core::lshift";
pub const RSHIFT: &str = "core::rshift";

pub const LEQ: &str = "core::leq";
pub const LT: &str = "core::lt";
pub const GEQ: &str = "core::geq";
pub const GT: &str = "core::gt";
pub const REFEQ: &str = "core::refeq";
pub const VALEQ: &str = "core::valeq";
pub const REFNEQ: &str = "core::refneq";
pub const VALNEQ: &str = "core::valneq";

pub const PIPE: &str = "core::pipe";
pub const RPIPE: &str = "core::rpipe";
pub const COMPOSE: &str = "core::compose";
pub const RCOMPOSE: &str = "core::rcompose";

pub const CONS: &str = "core::cons";

pub const RETURN: &str = "core::return";
pub const RESET: &str = "core::reset";
pub const END: &str = "core::end";

macro_rules! binop {
    ($builder:expr, $label:expr, $($op:expr),+) => {
        $builder
            .write_label($label.to_owned())
            .shift(RETURN)
            .write_instruction(Instruction::LoadLocal(0))
            .write_instruction(Instruction::Swap)
            $(.write_instruction($op))+
            .write_instruction(Instruction::Reset)
    };
}

macro_rules! binop_ {
    ($builder:expr, $label:expr, $($op:expr),+) => {
        $builder
            .write_label($label.to_owned())
            .shift(RETURN)
            .write_instruction(Instruction::LoadLocal(0))
            $(.write_instruction($op))+
            .write_instruction(Instruction::Reset)
    };
}

macro_rules! unop {
    ($builder:expr, $label:expr, $($op:expr),+) => {
        $builder
            .write_label($label.to_owned())
            $(.write_instruction($op))+
            .write_instruction(Instruction::Return)
    };
}

pub(crate) fn write_preamble(builder: &mut ProgramContext) {
    binop!(builder, ADD, Instruction::Add);
    binop!(builder, SUB, Instruction::Subtract);
    binop!(builder, MUL, Instruction::Multiply);
    binop!(builder, DIV, Instruction::Divide);
    binop!(builder, INTDIV, Instruction::IntDivide);
    binop!(builder, REM, Instruction::Remainder);
    binop!(builder, POW, Instruction::Power);
    unop!(builder, NEGATE, Instruction::Negate);

    binop!(builder, GLUE, Instruction::Glue);

    binop!(builder, AND, Instruction::And);
    binop!(builder, OR, Instruction::Or);
    unop!(builder, NOT, Instruction::Not);

    binop!(builder, BITAND, Instruction::BitwiseAnd);
    binop!(builder, BITOR, Instruction::BitwiseOr);
    binop!(builder, BITXOR, Instruction::BitwiseXor);
    unop!(builder, BITNEG, Instruction::BitwiseNeg);
    binop!(builder, LSHIFT, Instruction::LeftShift);
    binop!(builder, RSHIFT, Instruction::RightShift);

    binop!(builder, LEQ, Instruction::Leq);
    binop!(builder, LT, Instruction::Lt);
    binop!(builder, GEQ, Instruction::Geq);
    binop!(builder, GT, Instruction::Gt);
    binop!(builder, REFEQ, Instruction::RefEq);
    binop!(builder, VALEQ, Instruction::ValEq);
    binop!(builder, REFNEQ, Instruction::RefNeq);
    binop!(builder, VALNEQ, Instruction::ValNeq);

    binop!(builder, PIPE, Instruction::Call(1));
    binop_!(builder, RPIPE, Instruction::Call(1));

    binop!(builder, CONS, Instruction::Cons);

    builder
        .write_label(RCOMPOSE.to_owned())
        .shift(RETURN)
        .shift(RESET)
        .write_instruction(Instruction::LoadLocal(0))
        .write_instruction(Instruction::Swap)
        .write_instruction(Instruction::Call(1))
        .write_instruction(Instruction::LoadLocal(2))
        .write_instruction(Instruction::Swap)
        .write_instruction(Instruction::Call(1))
        .write_instruction(Instruction::Reset);
    builder
        .write_label(COMPOSE.to_owned())
        .shift(RETURN)
        .shift(RESET)
        .write_instruction(Instruction::LoadLocal(2))
        .write_instruction(Instruction::Swap)
        .write_instruction(Instruction::Call(1))
        .write_instruction(Instruction::LoadLocal(0))
        .write_instruction(Instruction::Swap)
        .write_instruction(Instruction::Call(1))
        .write_instruction(Instruction::Reset);

    builder
        .write_label(RESET.to_owned())
        .write_instruction(Instruction::Reset)
        .write_label(END.to_owned())
        .write_instruction(Instruction::Fizzle)
        .write_label(RETURN.to_owned())
        .write_instruction(Instruction::Return);
}
