use crate::{context::Context, INCORRECT_ARITY, INVALID_CALL};
use trilogy_ir::ir;
use trilogy_vm::{Instruction, Struct};

pub(crate) fn unapply_2(
    application: &ir::Application,
) -> (Option<&ir::Value>, &ir::Value, &ir::Value) {
    match &application.function.value {
        ir::Value::Application(lhs) => (
            Some(&lhs.function.value),
            &lhs.argument.value,
            &application.argument.value,
        ),
        _ => (
            None,
            &application.function.value,
            &application.argument.value,
        ),
    }
}

pub(crate) fn unlock_call(context: &mut Context, atom: &str, arity: usize) {
    context
        .instruction(Instruction::Destruct)
        .instruction(Instruction::Copy)
        .atom(atom)
        .instruction(Instruction::ValEq)
        .cond_jump(INVALID_CALL)
        .instruction(Instruction::Pop)
        .instruction(Instruction::Copy)
        .constant(arity)
        .instruction(Instruction::ValEq)
        .cond_jump(INCORRECT_ARITY)
        .instruction(Instruction::Pop);
}

#[inline(always)]
pub(crate) fn unlock_apply(context: &mut Context) {
    unlock_call(context, "function", 1);
}

pub(crate) fn call_procedure(context: &mut Context, arity: usize) {
    context
        .constant(arity)
        .atom("procedure")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Call(arity as u32 + 1));
}

pub(crate) fn apply_function(context: &mut Context) {
    let function = context.make_atom("function");
    context
        .constant(Struct::new(function, 1))
        .instruction(Instruction::Call(2));
}

pub(crate) fn apply_function_become(context: &mut Context) {
    let function = context.make_atom("function");
    context
        .constant(Struct::new(function, 1))
        .instruction(Instruction::Become(2));
}

pub(crate) fn apply_module(context: &mut Context) {
    let module = context.make_atom("module");
    context
        .constant(Struct::new(module, 1))
        .instruction(Instruction::Call(2));
}
