use crate::{write_evaluation, Context};
use trilogy_ir::ir;
use trilogy_vm::Instruction;

pub(crate) fn write_procedure(context: &mut Context, procedure: &ir::ProcedureDefinition) {
    let beginning = context.labeler.begin_procedure(&procedure.name);
    context.write_label(beginning).unwrap();
    let for_id = context.labeler.for_id(&procedure.name.id);
    context.write_label(for_id).unwrap();

    for (i, overload) in procedure.overloads.iter().enumerate() {
        let beginning = context.labeler.begin_overload(i);
        context.write_label(beginning).unwrap();
        write_procedure_overload(context, overload);
        let end = context.labeler.end();
        context.write_label(end).unwrap();
    }

    let end = context.labeler.end();
    context
        .write_label(end)
        .unwrap()
        .write_instruction(Instruction::Fizzle);
}

fn write_procedure_overload(context: &mut Context, procedure: &ir::Procedure) {
    for _parameter in &procedure.parameters {}
    write_evaluation(context, &procedure.body);
}
