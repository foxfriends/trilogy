use crate::context::Context;
use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_vm::Instruction;

pub(crate) fn write_rule(context: &mut Context, rule: &ir::Rule, on_fail: &str) {
    let entry = context.labeler.unique_hint("entry");
    context
        .write_instruction(Instruction::Copy)
        .write_instruction(Instruction::Const(().into()))
        .write_instruction(Instruction::ValEq)
        .cond_jump(&entry)
        .write_instruction(Instruction::Pop);
    write_query_state(context, &rule.body);
    context.write_label(entry.clone());
    write_query(context, &rule.body, on_fail, Some(1));
    context.scope.intermediate();
    for param in &rule.parameters {
        write_expression(context, param);
        context.scope.intermediate();
    }
    for _ in 1..rule.parameters.len() {
        context
            .write_instruction(Instruction::Swap)
            .write_instruction(Instruction::Cons);
        context.scope.end_intermediate();
    }
    // This ends with 2 expected values on the stack ([state, retval])
    // so they are no longer intemediate
    context.scope.end_intermediate();
    context.scope.end_intermediate();
}
