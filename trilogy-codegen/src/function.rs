use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_vm::Instruction;

pub(crate) fn write_function(context: &mut Context, function: &ir::Function) {
    let mut cleanup = vec![context.labeler.unique_hint("next")];
    for (i, parameter) in function.parameters.iter().enumerate() {
        context.declare_variables(parameter.bindings());
        context.write_instruction(Instruction::LoadLocal(i));
        write_pattern_match(context, parameter, &cleanup[i]);
        cleanup.push(context.labeler.unique_hint("cleanup"));
    }
    write_expression(context, &function.body);

    for parameter in function.parameters.iter().rev() {
        context.write_label(cleanup.pop().unwrap());
        context.undeclare_variables(parameter.bindings(), true);
    }
    context.write_label(cleanup.pop().unwrap());
}
