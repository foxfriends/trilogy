use crate::prelude::*;
use trilogy_ir::ir;
use trilogy_vm::Instruction;

// TODO: all the fizzles in here should probably actually be handled by an `on_fail`
// mechanism of some sort.
//
// may even need an `on_pass` mechanism as well, so that the natural continuation
// can be taken when "done".

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
            let element = context.labeler.unique_hint("in_elem");
            let continuation = context.labeler.unique_hint("in_cont");
            write_expression(context, &unification.expression);
            context
                .shift(&element)
                .jump(&continuation)
                .write_label(element)
                .unwrap();

            // TODO: iterate the iterator here

            let on_fail = context.labeler.unique_hint("unif_fail");
            let on_pass = context.labeler.unique_hint("unif_pass");

            context.write_label(continuation).unwrap();
            write_pattern_match(context, &unification.pattern, &on_fail);
            context
                .jump(&on_pass)
                .write_label(on_fail)
                .unwrap()
                .write_instruction(Instruction::Fizzle)
                .write_label(on_pass)
                .unwrap();
        }
        ir::QueryValue::Is(expr) => {
            let on_pass = context.labeler.unique_hint("on_pass");
            write_expression(context, expr);
            context
                .write_instruction(Instruction::Not)
                .cond_jump(&on_pass)
                .write_instruction(Instruction::Fizzle)
                .write_label(on_pass)
                .unwrap();
        }
        ir::QueryValue::End => {
            context.write_instruction(Instruction::Fizzle);
        }
        ir::QueryValue::Pass => { /* quietly succeed and just continue */ }
        value => todo!("{value:?}"),
    }
}
