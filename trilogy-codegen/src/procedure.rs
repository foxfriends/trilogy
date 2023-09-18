use crate::{preamble::END, prelude::*};
use trilogy_ir::ir;
use trilogy_ir::visitor::HasBindings;
use trilogy_vm::{Instruction, Value};

pub(crate) fn write_procedure(mut context: Context, procedure: &ir::Procedure) {
    for (offset, parameter) in procedure.parameters.iter().enumerate() {
        context.declare_variables(parameter.bindings());
        context.instruction(Instruction::LoadLocal(offset as u32));
        write_pattern_match(&mut context, parameter, END);
    }
    write_expression(&mut context, &procedure.body);
    context
        .instruction(Instruction::Const(Value::Unit))
        .instruction(Instruction::Return);
}
