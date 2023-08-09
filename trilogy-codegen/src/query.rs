use crate::{preamble::ITERATE_COLLECTION, prelude::*};
use std::collections::HashSet;
use trilogy_ir::visitor::HasBindings;
use trilogy_ir::{ir, Id};
use trilogy_vm::Instruction;

pub(crate) fn write_query_state(context: &mut Context, query: &ir::Query) {
    match &query.value {
        ir::QueryValue::Is(..)
        | ir::QueryValue::End
        | ir::QueryValue::Pass
        | ir::QueryValue::Not(..)
        | ir::QueryValue::Direct(..) => {
            context.write_instruction(Instruction::Const(true.into()));
        }
        ir::QueryValue::Lookup(lookup) => {
            write_expression(context, &lookup.path);
            context.write_instruction(Instruction::Call(0));
        }
        ir::QueryValue::Element(unification) => {
            write_expression(context, &unification.expression);
            context
                .write_procedure_reference(ITERATE_COLLECTION.to_owned())
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Call(1));
        }
        ir::QueryValue::Alternative(alt) => {
            write_query_state(context, &alt.0);
            context
                .write_instruction(Instruction::Const(().into()))
                .write_instruction(Instruction::Cons);
        }
        ir::QueryValue::Conjunction(alt)
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
    write_query_value(context, &query.value, &HashSet::default(), on_fail);
}

fn write_query_value(
    context: &mut Context,
    value: &ir::QueryValue,
    bound: &HashSet<Id>,
    on_fail: &str,
) {
    match &value {
        ir::QueryValue::Direct(unification) => {
            let vars = unification.pattern.bindings();
            let newly_bound = vars.difference(bound);
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Swap)
                .cond_jump(on_fail);
            write_expression(context, &unification.expression);
            unbind(context, newly_bound);
            write_pattern_match(context, &unification.pattern, on_fail);
        }
        ir::QueryValue::Element(unification) => {
            let cleanup = context.labeler.unique_hint("in_cleanup");
            let continuation = context.labeler.unique_hint("in_cont");

            let next = context.atom("next");
            let done = context.atom("done");

            let vars = unification.pattern.bindings();
            let newly_bound = vars.difference(bound);

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
            unbind(context, newly_bound);
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
        ir::QueryValue::Not(query) => {
            let on_pass = context.labeler.unique_hint("not_fail");
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Swap)
                .cond_jump(on_fail);
            context.scope.intermediate();
            write_query_state(context, query);
            write_query_value(context, &query.value, bound, &on_pass);
            context.write_instruction(Instruction::Pop).jump(on_fail);
            context
                .write_label(on_pass)
                .write_instruction(Instruction::Pop);
            context.scope.end_intermediate();
        }
        ir::QueryValue::Conjunction(conj) => {
            let out = context.labeler.unique_hint("conj_out");
            let next = context.labeler.unique_hint("conj_next");
            let cleanup = context.labeler.unique_hint("conj_cleanup");
            let outer = context.labeler.unique_hint("conj_outer");
            let inner = context.labeler.unique_hint("conj_inner");
            let reset = context.labeler.unique_hint("conj_reset");

            let lhs_vars = conj.0.bindings();
            let rhs_bound = bound.union(&lhs_vars).cloned().collect::<HashSet<_>>();

            context
                .write_instruction(Instruction::Uncons)
                .cond_jump(&outer);

            context
                .write_label(inner.clone())
                .write_instruction(Instruction::Uncons);
            context.scope.intermediate();
            write_query_value(context, &conj.1.value, &rhs_bound, &reset);
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
            write_query_value(context, &conj.0.value, bound, &cleanup);
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
        ir::QueryValue::Implication(imp) => {
            let out = context.labeler.unique_hint("impl_out");
            let cleanup_first = context.labeler.unique_hint("impl_cleanf");
            let cleanup_second = context.labeler.unique_hint("impl_cleans");
            let outer = context.labeler.unique_hint("impl_outer");
            let inner = context.labeler.unique_hint("impl_inner");

            let lhs_vars = imp.0.bindings();
            let rhs_bound = bound.union(&lhs_vars).cloned().collect::<HashSet<_>>();

            context
                .write_instruction(Instruction::Uncons)
                .cond_jump(&outer);

            context.write_label(inner.clone());
            write_query_value(context, &imp.1.value, &rhs_bound, &cleanup_second);
            context
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context.write_label(outer.clone());
            write_query_value(context, &imp.0.value, bound, &cleanup_first);
            context.write_instruction(Instruction::Pop);
            write_query_state(context, &imp.1);
            context.jump(&inner);

            context.write_label(cleanup_first);
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Cons)
                .jump(on_fail);
            context.write_label(cleanup_second);
            context
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(on_fail);

            context.write_label(out);
        }
        ir::QueryValue::Disjunction(disj) => {
            let first = context.labeler.unique_hint("disj_first");
            let second = context.labeler.unique_hint("disj_second");
            let next = context.labeler.unique_hint("disj_next");
            let out = context.labeler.unique_hint("disj_out");
            let cleanup = context.labeler.unique_hint("disj_cleanup");

            context
                .write_instruction(Instruction::Uncons)
                .cond_jump(&first);

            context.write_label(second.clone());
            write_query_value(context, &disj.1.value, bound, &cleanup);
            context
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context.write_label(first);
            write_query_value(context, &disj.0.value, bound, &next);
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context
                .write_label(next)
                .write_instruction(Instruction::Pop);
            write_query_state(context, &disj.1);
            context.jump(&second);

            context
                .write_label(cleanup)
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(on_fail);

            context.write_label(out);
        }
        ir::QueryValue::Alternative(alt) => {
            let maybe = context.labeler.unique_hint("alt_maybe");
            let second = context.labeler.unique_hint("alt_second");
            let out = context.labeler.unique_hint("alt_out");
            let cleanup_first = context.labeler.unique_hint("alt_cleanf");
            let cleanup_second = context.labeler.unique_hint("alt_cleans");

            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Swap);
            let is_uncommitted = context.scope.intermediate();

            context
                .write_instruction(Instruction::Uncons)
                .write_instruction(Instruction::Copy)
                .write_instruction(Instruction::Const(().into()))
                .write_instruction(Instruction::ValEq)
                .write_instruction(Instruction::SetLocal(is_uncommitted))
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::ValNeq)
                .cond_jump(&second);
            write_query_value(context, &alt.0.value, bound, &maybe);
            context
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context.write_label(second.clone());
            write_query_value(context, &alt.1.value, bound, &cleanup_second);
            context
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context
                .write_label(maybe)
                .write_instruction(Instruction::LoadLocal(is_uncommitted))
                .cond_jump(&cleanup_first)
                .write_instruction(Instruction::Pop);
            write_query_state(context, &alt.1);
            context.jump(&second);

            context
                .write_label(cleanup_first)
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Pop)
                .jump(on_fail);
            context
                .write_label(cleanup_second)
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Cons)
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Pop)
                .jump(on_fail);

            context
                .write_label(out)
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Pop);
            context.scope.end_intermediate();
        }
        ir::QueryValue::Lookup(_lookup) => todo!(),
    }
}

fn unbind<'a>(context: &mut Context, vars: impl IntoIterator<Item = &'a Id>) {
    for var in vars {
        match context.scope.lookup(var).unwrap() {
            Binding::Variable(index) => {
                context.write_instruction(Instruction::UnsetLocal(index));
            }
            _ => unreachable!(),
        }
    }
}
