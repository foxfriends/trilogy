use crate::{preamble::ITERATE_COLLECTION, prelude::*};
use trilogy_ir::ir;
use trilogy_vm::Instruction;

pub(crate) fn write_query_state(context: &mut Context, query: &ir::Query) {
    match &query.value {
        ir::QueryValue::Lookup(..)
        | ir::QueryValue::Is(..)
        | ir::QueryValue::Not(..)
        | ir::QueryValue::End
        | ir::QueryValue::Pass
        | ir::QueryValue::Direct(..) => {
            context.write_instruction(Instruction::Const(true.into()));
        }
        ir::QueryValue::Element(unification) => {
            write_expression(context, &unification.expression);
            context
                .write_procedure_reference(ITERATE_COLLECTION.to_owned())
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Call(1));
        }
        ir::QueryValue::Alternative(alt)
        | ir::QueryValue::Conjunction(alt)
        | ir::QueryValue::Implication(alt)
        | ir::QueryValue::Disjunction(alt) => {
            write_query_state(context, &alt.0);
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Cons);
        }
    }
}

/// Query expects the top of the stack to be the query's current state.
///
/// Upon finding some solution, the bindings have been set, and the new state is left on
/// the top of the stack.
///
/// On failure to find another solution, jump to `on_fail` with the state value left so
/// that all further requests will also fail.
///
/// It's up to the user of the query to call `write_query_state` to ensure that the
/// initial state is set, as well as to ensure that the state value is carried around
/// so that next time we enter the query it is set correctly.
pub(crate) fn write_query(context: &mut Context, query: &ir::Query, on_fail: &str) {
    match &query.value {
        ir::QueryValue::Direct(unification) => {
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Swap)
                .cond_jump(on_fail);
            write_expression(context, &unification.expression);
            write_pattern_match(context, &unification.pattern, on_fail);
        }
        ir::QueryValue::Element(unification) => {
            let cleanup = context.labeler.unique_hint("in_cleanup");
            let continuation = context.labeler.unique_hint("in_cont");

            let next = context.atom("next");
            let done = context.atom("done");

            context
                // Done if done
                .write_instruction(Instruction::Copy)
                .write_instruction(Instruction::Call(0))
                .write_instruction(Instruction::Copy)
                .write_instruction(Instruction::Const(done.into()))
                .write_instruction(Instruction::ValNeq)
                .cond_jump(&cleanup)
                // Runtime type error is probably expected if it's not an iterator when an
                // iterator is expected, but we just go to  fail instead anyway because it
                // seems easier for now. Maybe come back to that later, a panic instruction
                // is added or something.
                .write_instruction(Instruction::Copy)
                .write_instruction(Instruction::TypeOf)
                .write_instruction(Instruction::Const("struct".into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(&cleanup)
                .write_instruction(Instruction::Destruct)
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Const(next.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(&cleanup);
            write_pattern_match(context, &unification.pattern, on_fail);
            context
                .jump(&continuation)
                .write_label(cleanup)
                .write_instruction(Instruction::Pop)
                .jump(on_fail)
                .write_label(continuation);
        }
        ir::QueryValue::Is(expr) => {
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Swap)
                .cond_jump(on_fail);
            write_expression(context, expr);
            context.cond_jump(on_fail);
        }
        ir::QueryValue::End => {
            context.jump(on_fail);
        }
        ir::QueryValue::Pass => {
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Swap)
                .cond_jump(on_fail);
        }
        ir::QueryValue::Conjunction(conj) => {
            let out = context.labeler.unique_hint("conj_out");
            let next = context.labeler.unique_hint("conj_next");
            let cleanup = context.labeler.unique_hint("conj_cleanup");
            let outer = context.labeler.unique_hint("conj_outer");
            let inner = context.labeler.unique_hint("conj_inner");
            let reset = context.labeler.unique_hint("conj_reset");

            context
                .write_instruction(Instruction::Uncons)
                .cond_jump(&outer);

            context
                .write_label(inner.clone())
                .write_instruction(Instruction::Uncons);
            context.scope.intermediate();
            write_query(context, &conj.1, &reset);
            context.scope.end_intermediate();
            context
                .write_instruction(Instruction::Cons)
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context
                .write_label(reset)
                .write_instruction(Instruction::Pop)
                .write_instruction(Instruction::Reset);

            context.write_label(outer.clone());
            write_query(context, &conj.0, &cleanup);
            write_query_state(context, &conj.1);
            context
                .write_instruction(Instruction::Cons)
                .write_instruction(Instruction::SetRegister(1))
                .shift(&next)
                .jump(&inner);
            context
                .write_label(next)
                .write_instruction(Instruction::LoadRegister(1))
                .write_instruction(Instruction::Call(1))
                .jump(&outer);

            context
                .write_label(cleanup)
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Cons)
                .jump(on_fail);

            context.write_label(out);
        }
        // ir::QueryValue::Disjunction(disj) => {
        //     let cleanup = context.labeler.unique_hint("disj_cleanup");
        //     let next = context.labeler.unique_hint("disj_next");
        //     let done = context.labeler.unique_hint("disj_done");
        //     context
        //         .write_instruction(Instruction::Uncons)
        //         .write_instruction(Instruction::Swap);
        //     context.scope.intermediate();
        //     write_query(context, &disj.0, &next);
        //     context
        //         .write_instruction(Instruction::Swap)
        //         .write_instruction(Instruction::Cons)
        //         .jump(&done)
        //         .write_label(next)
        //         .write_instruction(Instruction::Swap);
        //     write_query(context, &disj.1, &cleanup);
        //     context.scope.end_intermediate();
        //     context
        //         .write_instruction(Instruction::Cons)
        //         .jump(&done)
        //         .write_label(cleanup)
        //         .write_instruction(Instruction::Pop)
        //         .jump(on_fail)
        //         .write_label(done);
        // }
        value => todo!("{value:?}"),
    }
}
