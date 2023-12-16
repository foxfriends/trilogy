use crate::{preamble::END, prelude::*};
use trilogy_ir::ir;
use trilogy_ir::visitor::HasBindings;
use trilogy_vm::{Instruction, Offset};

pub(crate) fn write_procedure(mut context: Context, procedure: &ir::Procedure) {
    for (offset, parameter) in procedure.parameters.iter().enumerate() {
        context.declare_variables(parameter.bindings());
        context
            .instruction(Instruction::LoadLocal(offset as Offset))
            .pattern_match(parameter, END);
    }
    context
        .evaluate(&procedure.body)
        .instruction(Instruction::Unit)
        .instruction(Instruction::Return);
}
