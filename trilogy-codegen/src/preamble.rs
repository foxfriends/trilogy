use super::prelude::*;
use crate::context::ProgramContext;
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

pub const ASSIGN: &str = "core::assign";
pub const ASSIGN_ANY: &str = "core::assign_any";
pub const ASSIGN_INT: &str = "core::assign_int";
pub const ACCESS: &str = "core::access";
pub const ACCESS_ANY: &str = "core::access_any";
pub const ACCESS_INT: &str = "core::access_int";

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
pub const END: &str = "core::end";
pub const YIELD: &str = "core::yield";
pub const EXIT: &str = "core::exit";

pub const RUNTIME_TYPE_ERROR: &str = "panic::runtime_type_error";
pub const INVALID_ACCESSOR: &str = "panic::invalid_accessor";
pub const INCORRECT_ARITY: &str = "panic::incorrect_arity";
pub const INVALID_CALL: &str = "panic::invalid_call";

pub const MIA: &str = "yield::MIA";

macro_rules! binop {
    ($builder:expr, $label:expr, $lty:expr, $rty:expr, $($op:expr),+) => {{
        $builder
            .label($label)
            .unlock_function()
            .close(RETURN)
            .unlock_function()
            .instruction(Instruction::LoadLocal(0))
            .typecheck($lty)
            .instruction(Instruction::Swap)
            .typecheck($rty)
            $(.instruction($op))+
            .instruction(Instruction::Return)
    }};
}

macro_rules! binop_ {
    ($builder:expr, $label:expr, $lty:expr, $rty:expr, $($op:expr),+) => {{
        $builder
            .label($label)
            .unlock_function()
            .close(RETURN)
            .unlock_function()
            .instruction(Instruction::LoadLocal(0))
            $(.instruction($op))+
            .instruction(Instruction::Return)
    }};
}

macro_rules! unop {
    ($builder:expr, $label:expr, $ty:expr, $($op:expr),+) => {{
        $builder
            .label($label)
            .unlock_function()
            .typecheck($ty)
            $(.instruction($op))+
            .instruction(Instruction::Return)
    }};
}

#[rustfmt::skip]
pub(crate) fn write_preamble(builder: &mut ProgramContext) {
    binop!(builder, ADD, "number", "number", Instruction::Add);
    binop!(builder, SUB, "number", "number", Instruction::Subtract);
    binop!(builder, MUL, "number", "number", Instruction::Multiply);
    binop!(builder, DIV, "number", "number", Instruction::Divide);
    binop!(builder, INTDIV, "number", "number", Instruction::IntDivide);
    binop!(builder, REM, "number", "number", Instruction::Remainder);
    binop!(builder, POW, "number", "number", Instruction::Power);
    unop!(builder, NEGATE, "number", Instruction::Negate);

    binop!(builder, GLUE, "string", "string", Instruction::Glue);

    binop!(builder, AND, "boolean", "boolean", Instruction::And);
    binop!(builder, OR, "boolean", "boolean", Instruction::Or);
    unop!(builder, NOT, "boolean", Instruction::Not);

    binop!(builder, BITAND, "bits", "bits", Instruction::BitwiseAnd);
    binop!(builder, BITOR, "bits", "bits", Instruction::BitwiseOr);
    binop!(builder, BITXOR, "bits", "bits", Instruction::BitwiseXor);
    unop!(builder, BITNEG, "bits", Instruction::BitwiseNeg);
    binop!(builder, LSHIFT, "bits", "number", Instruction::LeftShift);
    binop!(builder, RSHIFT, "bits", "number", Instruction::RightShift);

    binop!(builder, LEQ, &(), &(), Instruction::Leq);
    binop!(builder, LT, &(), &(), Instruction::Lt);
    binop!(builder, GEQ, &(), &(), Instruction::Geq);
    binop!(builder, GT, &(), &(), Instruction::Gt);
    binop!(builder, REFEQ, &(), &(), Instruction::RefEq);
    binop!(builder, VALEQ, &(), &(), Instruction::ValEq);
    binop!(builder, REFNEQ, &(), &(), Instruction::RefNeq);
    binop!(builder, VALNEQ, &(), &(), Instruction::ValNeq);

    binop!(builder, PIPE, "callable", &(), Instruction::Call(1));
    binop_!(builder, RPIPE, &(), "callable", Instruction::Call(1));

    binop!(builder, CONS, &(), &(), Instruction::Cons);

    builder
        .label(COMPOSE)
        .unlock_function()
        .typecheck("callable")
        .close(RETURN)
        .unlock_function()
        .typecheck("callable")
        .close(RETURN)
        .unlock_function()
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Swap)
        .call_function()
        .instruction(Instruction::LoadLocal(1))
        .instruction(Instruction::Swap)
        .call_function()
        .instruction(Instruction::Return);

    builder
        .label(RCOMPOSE)
        .unlock_function()
        .typecheck("callable")
        .close(RETURN)
        .unlock_function()
        .typecheck("callable")
        .close(RETURN)
        .unlock_function()
        .instruction(Instruction::LoadLocal(1))
        .instruction(Instruction::Swap)
        .call_function()
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Swap)
        .call_function()
        .instruction(Instruction::Return);

    builder
        .label(ASSIGN)
        .unlock_procedure(3)
        .instruction(Instruction::LoadLocal(0))
        .try_type("record", Ok(ASSIGN_ANY))
        .try_type("array", Ok(ASSIGN_INT))
        .atom("NotAccessible")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);
    builder
        .label(ASSIGN_ANY)
        .instruction(Instruction::Pop)
        .instruction(Instruction::Assign)
        .instruction(Instruction::Return);
    builder
        .label(ASSIGN_INT)
        .instruction(Instruction::Pop)
        .instruction(Instruction::LoadLocal(1))
        .typecheck("number")
        .instruction(Instruction::Copy)
        .constant(1)
        .instruction(Instruction::IntDivide)
        .instruction(Instruction::ValEq)
        .cond_jump(INVALID_ACCESSOR)
        .instruction(Instruction::Assign)
        .instruction(Instruction::Return);

    builder
        .label(ACCESS)
        .unlock_function()
        .try_type("record", Ok(ACCESS_ANY))
        .try_type("array", Ok(ACCESS_INT))
        .try_type("string", Ok(ACCESS_INT))
        .try_type("bits", Ok(ACCESS_INT))
        .atom("NotAccessible")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);
    builder
        .label(ACCESS_ANY)
        .close(RETURN)
        .unlock_function()
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::LoadLocal(1))
        .instruction(Instruction::Contains)
        .cond_jump(MIA)
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Swap)
        .instruction(Instruction::Access)
        .instruction(Instruction::Return);
    builder
        .label(ACCESS_INT)
        .close(RETURN)
        .unlock_function()
        .typecheck("number")
        .instruction(Instruction::Copy)
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Length)
        .instruction(Instruction::Lt)
        .cond_jump(MIA)
        .instruction(Instruction::Copy)
        .constant(0)
        .instruction(Instruction::Geq)
        .cond_jump(MIA)
        .instruction(Instruction::Copy)
        .instruction(Instruction::Copy)
        .constant(1)
        .instruction(Instruction::IntDivide)
        .instruction(Instruction::ValEq)
        .cond_jump(INVALID_ACCESSOR)
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::Swap)
        .instruction(Instruction::Access)
        .instruction(Instruction::Return);

    builder
        .label(MIA)
        .reference(YIELD)
        .atom("MIA")
        .become_function();

    builder
        .label(ITERATE_COLLECTION)
        .try_type("array", Ok(ITERATE_ARRAY))
        .try_type("set", Ok(ITERATE_SET))
        .try_type("record", Ok(ITERATE_RECORD))
        .try_type("tuple", Ok(ITERATE_LIST))
        .try_type("unit", Ok(ITERATE_LIST))
        .atom("NotIterable")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);

    builder
        .label(ITERATE_SET)
        .label(ITERATE_RECORD)
        .instruction(Instruction::Entries)
        .label(ITERATE_ARRAY)
        .constant(0)
        .repeat(|context, end| {
            context
                .instruction(Instruction::LoadLocal(0))
                .instruction(Instruction::Length)
                .instruction(Instruction::LoadLocal(1))
                .instruction(Instruction::Gt)
                .cond_jump(end)
                .instruction(Instruction::LoadLocal(0))
                .instruction(Instruction::LoadLocal(1))
                .instruction(Instruction::Access)
                .atom("next")
                .instruction(Instruction::Construct)
                .r#yield()
                .instruction(Instruction::Pop)
                .constant(1)
                .instruction(Instruction::Add);
        })
        .constant(())
        .instruction(Instruction::Return);

    builder
        .label(ITERATE_LIST)
        .repeat(|context, end| {
            context
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(Value::Unit))
                .instruction(Instruction::ValNeq)
                .cond_jump(end)
                .typecheck("tuple")
                .instruction(Instruction::Uncons)
                .instruction(Instruction::Swap)
                .atom("next")
                .instruction(Instruction::Construct)
                .r#yield()
                .instruction(Instruction::Pop);
        })
        .instruction(Instruction::Return);

    builder
        .label(END)
        .instruction(Instruction::Fizzle)
        .label(RETURN)
        .instruction(Instruction::Return)
        .label(EXIT)
        .instruction(Instruction::Exit);

    let yielding = builder.make_label("yielding");
    let no_handler = builder.make_label("no_handler");

    builder
        .label(YIELD)
        .unlock_function()
        .instruction(Instruction::LoadRegister(HANDLER))
        .instruction(Instruction::Const(Value::Unit))
        .instruction(Instruction::ValNeq)
        .cond_jump(&no_handler)
        // Save the module context and handler to restore after resuming
        .instruction(Instruction::LoadRegister(MODULE))
        .instruction(Instruction::Swap)
        // The handler is also about to be called, so it goes second
        .instruction(Instruction::LoadRegister(HANDLER))
        .instruction(Instruction::Swap)
        .shift(&yielding)
        // This is where we go when "resumed"
        .unlock_function()
        // Restore the context and previous hadler
        .instruction(Instruction::LoadLocal(0))
        .instruction(Instruction::SetRegister(MODULE))
        .instruction(Instruction::LoadLocal(1))
        .instruction(Instruction::SetRegister(HANDLER))
        // Then the `yield` "returns" the resumed value
        .instruction(Instruction::Return)
        .label(yielding)
        // Call the handler with the effect, then the "resume" continuation
        .instruction(Instruction::Become(2))
        .label(no_handler)
        .atom("UnhandledEffect")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);

    builder
        .label(INVALID_ACCESSOR)
        .atom("InvalidAccessor")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);

    builder
        .label(INCORRECT_ARITY)
        .atom("IncorrectArity")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);

    builder
        .label(INVALID_CALL)
        .atom("InvalidCall")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);

    builder
        .label(RUNTIME_TYPE_ERROR)
        .atom("RuntimeTypeError")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Panic);
}
