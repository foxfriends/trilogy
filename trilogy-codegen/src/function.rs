use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_ir::visitor::HasBindings;
use trilogy_vm::{Instruction, Offset};

pub(crate) fn write_function(context: &mut Context, function: &ir::Function) {
    let mut cleanup = vec![context.make_label("next")];
    for (i, parameter) in function.parameters.iter().enumerate() {
        context.declare_variables(parameter.bindings());
        context.instruction(Instruction::LoadLocal(i as Offset));
        cleanup.push(context.make_label("cleanup"));
        context.pattern_match(parameter, &cleanup[i + 1]);
    }
    context.evaluate(&function.body);
    for parameter in function.parameters.iter().rev() {
        context.label(cleanup.pop().unwrap());
        context.undeclare_variables(parameter.bindings(), true);
    }
    context.label(cleanup.pop().unwrap());
}
