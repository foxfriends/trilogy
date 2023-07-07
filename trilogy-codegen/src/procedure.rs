use crate::{write_evaluation, Labeler};
use trilogy_ir::ir;
use trilogy_vm::{Instruction, ProgramBuilder};

pub(crate) fn write_procedure(
    labeler: &mut Labeler,
    builder: &mut ProgramBuilder,
    procedure: &ir::ProcedureDefinition,
) {
    builder
        .write_label(labeler.begin_procedure(&procedure.name))
        .unwrap();

    for (i, overload) in procedure.overloads.iter().enumerate() {
        builder.write_label(labeler.begin_overload(i)).unwrap();
        write_procedure_overload(labeler, builder, overload);
        builder.write_label(labeler.end()).unwrap();
    }
    builder
        .write_label(labeler.end())
        .unwrap()
        .write_instruction(Instruction::Fizzle);
}

fn write_procedure_overload(
    labeler: &mut Labeler,
    builder: &mut ProgramBuilder,
    procedure: &ir::Procedure,
) {
    for _parameter in &procedure.parameters {}
    write_evaluation(labeler, builder, &procedure.body);
}
