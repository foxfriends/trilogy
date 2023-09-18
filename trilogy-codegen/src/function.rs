use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_ir::visitor::HasBindings;
use trilogy_vm::Instruction;

pub(crate) fn write_function(context: &mut Context, function: &ir::Function) {
    let mut cleanup = vec![context.labeler.unique_hint("next")];
    for (i, parameter) in function.parameters.iter().enumerate() {
        context.declare_variables(parameter.bindings());
        context.instruction(Instruction::LoadLocal(i as u32));
        write_pattern_match(context, parameter, &cleanup[i]);
        cleanup.push(context.labeler.unique_hint("cleanup"));
    }
    write_expression(context, &function.body);

    for parameter in function.parameters.iter().rev() {
        context.label(cleanup.pop().unwrap());
        context.undeclare_variables(parameter.bindings(), true);
    }
    context.label(cleanup.pop().unwrap());
}
