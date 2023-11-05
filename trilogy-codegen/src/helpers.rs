use crate::{context::Context, INCORRECT_ARITY, INVALID_CALL};
use trilogy_ir::ir;
use trilogy_vm::Instruction;

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
        .instruction(Instruction::Copy)
        .instruction(Instruction::Destruct)
        .instruction(Instruction::Copy)
        .atom(atom)
        .instruction(Instruction::ValEq)
        .cond_jump(INVALID_CALL)
        .instruction(Instruction::Pop)
        .constant(arity)
        .instruction(Instruction::ValEq)
        .cond_jump(INCORRECT_ARITY)
        .instruction(Instruction::Pop);
}

pub(crate) fn call_procedure(context: &mut Context, arity: usize) {
    context
        .constant(arity)
        .atom("procedure")
        .instruction(Instruction::Construct)
        .instruction(Instruction::Call(arity as u32 + 1));
}
