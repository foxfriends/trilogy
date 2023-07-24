use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_vm::{Instruction, Value};

pub(crate) fn write_procedure(mut context: Context, procedure: &ir::Procedure) {
    let on_fail = context.labeler.unique_hint("proc_fail");
    for (offset, parameter) in procedure.parameters.iter().enumerate() {
        context.declare_variables(parameter.bindings());
        context.write_instruction(Instruction::LoadLocal(offset));
        write_pattern_match(&mut context, parameter, &on_fail);
    }
    write_expression(&mut context, &procedure.body);
    context
        .write_instruction(Instruction::Const(Value::Unit))
        .write_instruction(Instruction::Return)
        .write_label(on_fail)
        .unwrap()
        .write_instruction(Instruction::Fizzle);
}
