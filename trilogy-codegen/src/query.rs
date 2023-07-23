use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_vm::Instruction;

pub(crate) fn write_query(context: &mut Context, query: &ir::Query) {
    match &query.value {
        ir::QueryValue::Direct(unification) => {
            write_expression(context, &unification.expression);
            let on_fail = context.labeler.unique_hint("unif_fail");
            let on_pass = context.labeler.unique_hint("unif_pass");
            write_pattern_match(context, &unification.pattern, &on_fail);
            context
                .jump(&on_pass)
                .write_label(on_fail)
                .unwrap()
                .write_instruction(Instruction::Fizzle)
                .write_label(on_pass)
                .unwrap();
        }
        ir::QueryValue::Element(unification) => {
            write_expression(context, &unification.expression);
            let on_fail = context.labeler.unique_hint("unif_fail");
            let on_pass = context.labeler.unique_hint("unif_pass");
            write_pattern_match(context, &unification.pattern, &on_fail);
            context
                .jump(&on_pass)
                .write_label(on_fail)
                .unwrap()
                .write_instruction(Instruction::Fizzle)
                .write_label(on_pass)
                .unwrap();
        }
        value => todo!("{value:?}"),
    }
}
