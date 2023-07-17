use crate::{write_evaluation, write_pattern_match, Context};
use trilogy_ir::ir;
use trilogy_vm::{Instruction, Value};

pub(crate) fn write_procedure(context: &mut Context, procedure: &ir::ProcedureDefinition) {
    let beginning = context.labeler.begin_procedure(&procedure.name);
    context.write_label(beginning).unwrap();
    let for_id = context.labeler.for_id(&procedure.name.id);
    context.write_label(for_id).unwrap();
    for overload in &procedure.overloads {
        let on_fail = context.labeler.unique();
        write_procedure_overload(context, overload, &on_fail);
        context.write_label(on_fail).unwrap();
    }
    context.write_instruction(Instruction::Fizzle);
}

fn write_procedure_overload(context: &mut Context, procedure: &ir::Procedure, on_fail: &str) {
    let mut offset = procedure.parameters.len();
    for binding in procedure.bindings() {
        if context.scope.lookup(&binding).is_none() {
            context.scope.declare_variable(binding, offset);
            context.write_instruction(Instruction::Const(Value::Unit));
            offset += 1;
        }
    }
    for (offset, parameter) in procedure.parameters.iter().enumerate() {
        context.write_instruction(Instruction::LoadLocal(offset));
        write_pattern_match(context, parameter, on_fail);
    }
    write_evaluation(context, &procedure.body);
    context
        .write_instruction(Instruction::Const(Value::Unit))
        .write_instruction(Instruction::Return);
}
