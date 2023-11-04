use crate::entrypoint::ProgramContext;
use trilogy_vm::{Instruction, Value};

pub const ADD: &str = "core::add";
pub const SUB: &str = "core::sub";
pub const MUL: &str = "core::mul";
pub const DIV: &str = "core::div";
pub const INTDIV: &str = "core::intdiv";
pub const REM: &str = "core::rem";
pub const POW: &str = "core::pow";
pub const NEGATE: &str = "core::neg";

pub const GLUE: &str = "core::glue";
pub const ACCESS: &str = "core::access";

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

pub const ITERATE_COLLECTION: &str = "core::iter";
pub const ITERATE_ARRAY: &str = "core::iter_array";
pub const ITERATE_SET: &str = "core::iter_set";
pub const ITERATE_RECORD: &str = "core::iter_record";
pub const ITERATE_LIST: &str = "core::iter_list";

pub const RETURN: &str = "core::return";
pub const RESET: &str = "core::reset";
pub const END: &str = "core::end";
pub const YIELD: &str = "core::yield";
pub const EXIT: &str = "core::exit";

pub(crate) const INVALID_ITERATOR: &str = "panic::invalid_iterator";

macro_rules! binop {
    ($builder:expr, $label:expr, $($op:expr),+) => {
        $builder
            .label($label)
            .shift(RETURN)
            .instruction(Instruction::LoadLocal(0))
            .instruction(Instruction::Swap)
            $(.instruction($op))+
            .instruction(Instruction::Reset)
    };
}

macro_rules! binop_ {
    ($builder:expr, $label:expr, $($op:expr),+) => {
        $builder
            .label($label)
            .shift(RETURN)
            .instruction(Instruction::LoadLocal(0))
            $(.instruction($op))+
            .instruction(Instruction::Reset)
    };
}

macro_rules! unop {
    ($builder:expr, $label:expr, $($op:expr),+) => {
        $builder
            .label($label)
            $(.instruction($op))+
            .instruction(Instruction::Return)
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
    binop!(builder, ACCESS, Instruction::Access);

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
        .label(RCOMPOSE)
        .close(RETURN)
        .close(RETURN)
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Swap)
        .instruction(Instruction::Call(1))
        .instruction(Instruction::LoadLocal(1))
        .instruction(Instruction::Swap)
        .instruction(Instruction::Call(1))
        .instruction(Instruction::Return);

    builder
        .label(COMPOSE)
        .close(RETURN)
        .close(RETURN)
        .instruction(Instruction::LoadLocal(1))
        .instruction(Instruction::Swap)
        .instruction(Instruction::Call(1))
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Swap)
        .instruction(Instruction::Call(1))
        .instruction(Instruction::Return);

    let callable = builder.atom("callable");
    let array = builder.atom("array");
    let set = builder.atom("set");
    let record = builder.atom("record");
    let tuple = builder.atom("tuple");

    let not_iterable = builder.atom("NotIterable");
    builder
        .label(ITERATE_COLLECTION)
        .instruction(Instruction::Copy)
        .instruction(Instruction::TypeOf)
        .instruction(Instruction::Const(callable.into()))
        .instruction(Instruction::ValNeq)
        .cond_jump(RETURN) // already an iterator (probably)
        .instruction(Instruction::Copy)
        .instruction(Instruction::TypeOf)
        .instruction(Instruction::Const(array.into()))
        .instruction(Instruction::ValNeq)
        .cond_jump(ITERATE_ARRAY)
        .instruction(Instruction::Copy)
        .instruction(Instruction::TypeOf)
        .instruction(Instruction::Const(set.into()))
        .instruction(Instruction::ValNeq)
        .cond_jump(ITERATE_SET)
        .instruction(Instruction::Copy)
        .instruction(Instruction::TypeOf)
        .instruction(Instruction::Const(record.into()))
        .instruction(Instruction::ValNeq)
        .cond_jump(ITERATE_RECORD)
        .instruction(Instruction::Copy)
        .instruction(Instruction::TypeOf)
        .instruction(Instruction::Const(tuple.into()))
        .instruction(Instruction::ValNeq)
        .cond_jump(ITERATE_LIST)
        .instruction(Instruction::Copy)
        .instruction(Instruction::Const(Value::Unit))
        .instruction(Instruction::ValNeq)
        .cond_jump(ITERATE_LIST)
        .instruction(Instruction::Const(not_iterable.into()))
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);

    let iter_done = builder.labeler.unique_hint("iter_done");
    let next = builder.atom("next");
    builder
        .label(ITERATE_SET)
        .label(ITERATE_RECORD)
        .instruction(Instruction::Entries)
        .label(ITERATE_ARRAY)
        .instruction(Instruction::Const(0.into()))
        .instruction(Instruction::Cons)
        .close(RETURN)
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Uncons)
        .instruction(Instruction::Copy)
        .instruction(Instruction::LoadLocal(1))
        .instruction(Instruction::Length)
        .instruction(Instruction::Lt)
        .cond_jump(&iter_done)
        .instruction(Instruction::Access)
        .instruction(Instruction::Const(next.clone().into()))
        .instruction(Instruction::Construct)
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Uncons)
        .instruction(Instruction::Const(1.into()))
        .instruction(Instruction::Add)
        .instruction(Instruction::Cons)
        .instruction(Instruction::SetLocal(0))
        .instruction(Instruction::Return);

    builder
        .label(ITERATE_LIST)
        .close(RETURN)
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Copy)
        .instruction(Instruction::Const(Value::Unit))
        .instruction(Instruction::ValNeq)
        .cond_jump(&iter_done)
        .instruction(Instruction::Uncons)
        .instruction(Instruction::SetLocal(0))
        .instruction(Instruction::Const(next.into()))
        .instruction(Instruction::Construct)
        .instruction(Instruction::Return);

    let done = builder.atom("done");
    builder
        .label(iter_done)
        .instruction(Instruction::Const(done.into()))
        .instruction(Instruction::Return);

    builder
        .label(RESET)
        .instruction(Instruction::Reset)
        .label(END)
        .instruction(Instruction::Fizzle)
        .label(RETURN)
        .instruction(Instruction::Return)
        .label(EXIT)
        .instruction(Instruction::Exit);

    let yielding = builder.labeler.unique_hint("yielding");
    let no_handler = builder.labeler.unique_hint("no_handler");
    let unhandled_effect = builder.atom("UnhandledEffect");

    builder
        .label(YIELD)
        .instruction(Instruction::LoadRegister(0))
        .instruction(Instruction::Const(Value::Unit))
        .instruction(Instruction::ValNeq)
        .cond_jump(&no_handler)
        .instruction(Instruction::LoadRegister(0))
        .instruction(Instruction::Swap)
        .shift(&yielding)
        // This is where we go when "resumed"
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::SetRegister(0))
        .instruction(Instruction::Return)
        .label(yielding)
        .instruction(Instruction::Become(2))
        .label(no_handler)
        .instruction(Instruction::Const(unhandled_effect.into()))
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);

    let invalid_iterator = builder.atom("InvalidIterator");
    builder
        .label(INVALID_ITERATOR)
        .instruction(Instruction::Const(invalid_iterator.into()))
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);
}
