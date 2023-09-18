use crate::{preamble::ITERATE_COLLECTION, prelude::*};
use std::collections::HashSet;
use trilogy_ir::visitor::{HasBindings, HasReferences};
use trilogy_ir::{ir, Id};
use trilogy_vm::Instruction;

pub(crate) fn write_query_state(context: &mut Context, query: &ir::Query) {
    match &query.value {
        ir::QueryValue::Is(..)
        | ir::QueryValue::End
        | ir::QueryValue::Pass
        | ir::QueryValue::Not(..)
        | ir::QueryValue::Direct(..) => {
            context.instruction(Instruction::Const(true.into()));
        }
        ir::QueryValue::Lookup(lookup) => {
            write_expression(context, &lookup.path);
            context
                .instruction(Instruction::Call(0))
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons);
        }
        ir::QueryValue::Element(unification) => {
            write_expression(context, &unification.expression);
            context
                .write_procedure_reference(ITERATE_COLLECTION.to_owned())
                .instruction(Instruction::Swap)
                .instruction(Instruction::Call(1));
        }
        ir::QueryValue::Alternative(alt) => {
            write_query_state(context, &alt.0);
            context
                .instruction(Instruction::Const(().into()))
                .instruction(Instruction::Cons);
        }
        ir::QueryValue::Conjunction(alt)
        | ir::QueryValue::Implication(alt)
        | ir::QueryValue::Disjunction(alt) => {
            write_query_state(context, &alt.0);
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons);
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
pub(crate) fn write_query(
    context: &mut Context,
    query: &ir::Query,
    on_fail: &str,
    runtime_bindset: Option<u32>,
) {
    write_query_value(
        context,
        &query.value,
        on_fail,
        Bindings {
            compile_time: &HashSet::default(),
            run_time: runtime_bindset,
        },
    );
}

#[derive(Copy, Clone)]
pub(crate) struct Bindings<'a> {
    compile_time: &'a HashSet<Id>,
    run_time: Option<u32>,
}

impl Bindings<'_> {
    fn is_bound(&self, var: &Id) -> bool {
        self.compile_time.contains(var)
    }
}

fn write_query_value(
    context: &mut Context,
    value: &ir::QueryValue,
    on_fail: &str,
    bound: Bindings<'_>,
) {
    match &value {
        ir::QueryValue::Direct(unification) => {
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
            write_expression(context, &unification.expression);
            unbind(context, bound, unification.pattern.bindings());
            write_pattern_match(context, &unification.pattern, on_fail);
        }
        ir::QueryValue::Element(unification) => {
            let cleanup = context.labeler.unique_hint("in_cleanup");
            let continuation = context.labeler.unique_hint("in_cont");

            let next = context.atom("next");
            let done = context.atom("done");

            context
                // Done if done
                .instruction(Instruction::Copy)
                .instruction(Instruction::Call(0))
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(done.into()))
                .instruction(Instruction::ValNeq)
                .cond_jump(&cleanup)
                // Runtime type error is probably expected if it's not an iterator when an
                // iterator is expected, but we just go to  fail instead anyway because it
                // seems easier for now. Maybe come back to that later, a panic instruction
                // is added or something.
                .instruction(Instruction::Copy)
                .instruction(Instruction::TypeOf)
                .instruction(Instruction::Const("struct".into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&cleanup)
                .instruction(Instruction::Destruct)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Const(next.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&cleanup);
            unbind(context, bound, unification.pattern.bindings());
            write_pattern_match(context, &unification.pattern, on_fail);
            context
                .jump(&continuation)
                .label(cleanup)
                .instruction(Instruction::Pop)
                .jump(on_fail)
                .label(continuation);
        }
        ir::QueryValue::Is(expr) => {
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
            write_expression(context, expr);
            context.cond_jump(on_fail);
        }
        ir::QueryValue::End => {
            context.jump(on_fail);
        }
        ir::QueryValue::Pass => {
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
        }
        ir::QueryValue::Not(query) => {
            let on_pass = context.labeler.unique_hint("not_fail");
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
            context.scope.intermediate();
            write_query_state(context, query);
            write_query_value(context, &query.value, &on_pass, bound);
            context.instruction(Instruction::Pop).jump(on_fail);
            context.label(on_pass).instruction(Instruction::Pop);
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
            let rhs_bound = bound
                .compile_time
                .union(&lhs_vars)
                .cloned()
                .collect::<HashSet<_>>();
            let rhs_bound = Bindings {
                compile_time: &rhs_bound,
                run_time: bound.run_time,
            };

            context.instruction(Instruction::Uncons).cond_jump(&outer);

            context
                .label(inner.clone())
                .instruction(Instruction::Uncons);
            context.scope.intermediate();
            write_query_value(context, &conj.1.value, &reset, rhs_bound);
            context.scope.end_intermediate();
            context
                .instruction(Instruction::Cons)
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(&out);

            context
                .label(reset)
                .instruction(Instruction::Pop)
                .instruction(Instruction::Reset);

            context.label(outer.clone());
            write_query_value(context, &conj.0.value, &cleanup, bound);
            write_query_state(context, &conj.1);
            context
                .instruction(Instruction::Cons)
                .instruction(Instruction::SetRegister(1))
                .shift(&next)
                .jump(&inner);
            context
                .label(next)
                .instruction(Instruction::LoadRegister(1))
                .instruction(Instruction::Call(1))
                .jump(&outer);

            context
                .label(cleanup)
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons)
                .jump(on_fail);

            context.label(out);
        }
        ir::QueryValue::Implication(imp) => {
            let out = context.labeler.unique_hint("impl_out");
            let cleanup_first = context.labeler.unique_hint("impl_cleanf");
            let cleanup_second = context.labeler.unique_hint("impl_cleans");
            let outer = context.labeler.unique_hint("impl_outer");
            let inner = context.labeler.unique_hint("impl_inner");

            let lhs_vars = imp.0.bindings();
            let rhs_bound = bound
                .compile_time
                .union(&lhs_vars)
                .cloned()
                .collect::<HashSet<_>>();
            let rhs_bound = Bindings {
                compile_time: &rhs_bound,
                run_time: bound.run_time,
            };

            context.instruction(Instruction::Uncons).cond_jump(&outer);

            context.label(inner.clone());
            write_query_value(context, &imp.1.value, &cleanup_second, rhs_bound);
            context
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(&out);

            context.label(outer.clone());
            write_query_value(context, &imp.0.value, &cleanup_first, bound);
            context.instruction(Instruction::Pop);
            write_query_state(context, &imp.1);
            context.jump(&inner);

            context.label(cleanup_first);
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons)
                .jump(on_fail);
            context.label(cleanup_second);
            context
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(on_fail);

            context.label(out);
        }
        ir::QueryValue::Disjunction(disj) => {
            let first = context.labeler.unique_hint("disj_first");
            let second = context.labeler.unique_hint("disj_second");
            let next = context.labeler.unique_hint("disj_next");
            let out = context.labeler.unique_hint("disj_out");
            let cleanup = context.labeler.unique_hint("disj_cleanup");

            context.instruction(Instruction::Uncons).cond_jump(&first);

            context.label(second.clone());
            write_query_value(context, &disj.1.value, &cleanup, bound);
            context
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(&out);

            context.label(first);
            write_query_value(context, &disj.0.value, &next, bound);
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons)
                .jump(&out);

            context.label(next).instruction(Instruction::Pop);
            write_query_state(context, &disj.1);
            context.jump(&second);

            context
                .label(cleanup)
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(on_fail);

            context.label(out);
        }
        ir::QueryValue::Alternative(alt) => {
            let maybe = context.labeler.unique_hint("alt_maybe");
            let second = context.labeler.unique_hint("alt_second");
            let out = context.labeler.unique_hint("alt_out");
            let cleanup_first = context.labeler.unique_hint("alt_cleanf");
            let cleanup_second = context.labeler.unique_hint("alt_cleans");

            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap);
            let is_uncommitted = context.scope.intermediate();

            context
                .instruction(Instruction::Uncons)
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(().into()))
                .instruction(Instruction::ValEq)
                .instruction(Instruction::SetLocal(is_uncommitted))
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::ValNeq)
                .cond_jump(&second);
            write_query_value(context, &alt.0.value, &maybe, bound);
            context
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(&out);

            context.label(second.clone());
            write_query_value(context, &alt.1.value, &cleanup_second, bound);
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons)
                .jump(&out);

            context
                .label(maybe)
                .instruction(Instruction::LoadLocal(is_uncommitted))
                .cond_jump(&cleanup_first)
                .instruction(Instruction::Pop);
            write_query_state(context, &alt.1);
            context.jump(&second);

            context
                .label(cleanup_first)
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Pop)
                .jump(on_fail);
            context
                .label(cleanup_second)
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Pop)
                .jump(on_fail);

            context
                .label(out)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Pop);
            context.scope.end_intermediate();
        }
        ir::QueryValue::Lookup(lookup) => {
            let setup = context.labeler.unique_hint("setup");
            let enter = context.labeler.unique_hint("enter");
            let cleanup = context.labeler.unique_hint("cleanup");
            let end = context.labeler.unique_hint("end");

            let next = context.atom("next");
            let done = context.atom("done");

            context.scope.intermediate();
            context
                .instruction(Instruction::Uncons)
                .cond_jump(&setup)
                .label(enter.clone())
                .instruction(Instruction::Copy)
                .instruction(Instruction::Call(0))
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(done.into()))
                .instruction(Instruction::ValNeq)
                .cond_jump(&cleanup)
                .instruction(Instruction::Destruct)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Const(next.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&cleanup);
            context.scope.intermediate();
            for pattern in &lookup.patterns {
                context.instruction(Instruction::Uncons);
                unbind(context, bound, pattern.bindings());
                write_pattern_match(context, pattern, &cleanup);
            }
            context.scope.end_intermediate();
            context
                .instruction(Instruction::Pop)
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(&end)
                .label(cleanup)
                .instruction(Instruction::Pop)
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(on_fail);

            context.label(setup);
            // We can predetermine all the statically assigned expressions
            let mut set = HashSet::new();
            for (i, pattern) in lookup.patterns.iter().enumerate() {
                let vars = pattern.references();
                if vars.iter().all(|var| bound.is_bound(var)) {
                    set.insert(i.into());
                }
            }
            context.instruction(Instruction::Const(set.into()));
            let save_into = context.scope.intermediate();
            for (i, pattern) in lookup.patterns.iter().enumerate() {
                pattern.bindings();
                evaluate(context, bound, pattern, i as u32, save_into);
                context.scope.intermediate();
            }

            context.instruction(Instruction::Call(lookup.patterns.len() as u32 + 1));
            for _ in &lookup.patterns {
                context.scope.end_intermediate();
            }
            context.scope.end_intermediate();
            context.jump(&enter).label(end);
        }
    }
}

fn unbind(context: &mut Context, bindset: Bindings<'_>, vars: HashSet<Id>) {
    let newly_bound = vars.difference(bindset.compile_time);
    for var in newly_bound {
        match context.scope.lookup(var).unwrap() {
            Binding::Variable(index) => {
                let skip = context.labeler.unique_hint("skip");
                if let Some(bindings) = bindset.run_time {
                    context
                        .instruction(Instruction::LoadLocal(bindings))
                        .instruction(Instruction::Const(index.into()))
                        .instruction(Instruction::Contains)
                        .instruction(Instruction::Not)
                        .jump(&skip);
                }
                context
                    .instruction(Instruction::UnsetLocal(index))
                    .label(skip);
            }
            _ => unreachable!(),
        }
    }
}

fn evaluate(
    context: &mut Context,
    bindset: Bindings<'_>,
    value: &ir::Expression,
    expr_index: u32,
    save_into: u32,
) {
    let vars = value.references();
    if vars.iter().all(|var| bindset.is_bound(var)) {
        write_expression(context, value);
    } else if bindset.run_time.is_some() {
        let nope = context.labeler.unique_hint("nope");
        for var in &vars {
            if bindset.is_bound(var) {
                continue;
            }
            match context.scope.lookup(var).unwrap() {
                Binding::Variable(index) => {
                    let runtime_bound = bindset.run_time.unwrap();
                    context
                        .instruction(Instruction::LoadLocal(runtime_bound))
                        .instruction(Instruction::Const(index.into()))
                        .instruction(Instruction::Contains)
                        .cond_jump(&nope);
                }
                _ => unreachable!(),
            }
        }
        let next = context.labeler.unique_hint("next");
        write_expression(context, value);
        context
            .instruction(Instruction::LoadLocal(save_into))
            .instruction(Instruction::Const(expr_index.into()))
            .instruction(Instruction::Insert)
            .instruction(Instruction::Pop)
            .jump(&next)
            .label(nope)
            .instruction(Instruction::Const(().into()))
            .label(next);
    } else {
        context.instruction(Instruction::Const(().into()));
    }
}
