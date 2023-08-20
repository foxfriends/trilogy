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
            context.write_instruction(Instruction::Const(true.into()));
        }
        ir::QueryValue::Lookup(lookup) => {
            write_expression(context, &lookup.path);
            context
                .write_instruction(Instruction::Call(0))
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Cons);
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
pub(crate) fn write_query(
    context: &mut Context,
    query: &ir::Query,
    on_fail: &str,
    runtime_bindset: Option<usize>,
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
    run_time: Option<usize>,
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
                .write_instruction(Instruction::Const(false.into()))
                .write_instruction(Instruction::Swap)
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
            unbind(context, bound, unification.pattern.bindings());
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
            write_query_value(context, &query.value, &on_pass, bound);
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
            let rhs_bound = bound
                .compile_time
                .union(&lhs_vars)
                .cloned()
                .collect::<HashSet<_>>();
            let rhs_bound = Bindings {
                compile_time: &rhs_bound,
                run_time: bound.run_time,
            };

            context
                .write_instruction(Instruction::Uncons)
                .cond_jump(&outer);

            context
                .write_label(inner.clone())
                .write_instruction(Instruction::Uncons);
            context.scope.intermediate();
            write_query_value(context, &conj.1.value, &reset, rhs_bound);
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
            write_query_value(context, &conj.0.value, &cleanup, bound);
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
            let rhs_bound = bound
                .compile_time
                .union(&lhs_vars)
                .cloned()
                .collect::<HashSet<_>>();
            let rhs_bound = Bindings {
                compile_time: &rhs_bound,
                run_time: bound.run_time,
            };

            context
                .write_instruction(Instruction::Uncons)
                .cond_jump(&outer);

            context.write_label(inner.clone());
            write_query_value(context, &imp.1.value, &cleanup_second, rhs_bound);
            context
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context.write_label(outer.clone());
            write_query_value(context, &imp.0.value, &cleanup_first, bound);
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
            write_query_value(context, &disj.1.value, &cleanup, bound);
            context
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context.write_label(first);
            write_query_value(context, &disj.0.value, &next, bound);
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
            write_query_value(context, &alt.0.value, &maybe, bound);
            context
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&out);

            context.write_label(second.clone());
            write_query_value(context, &alt.1.value, &cleanup_second, bound);
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
        ir::QueryValue::Lookup(lookup) => {
            let setup = context.labeler.unique_hint("setup");
            let enter = context.labeler.unique_hint("enter");
            let cleanup = context.labeler.unique_hint("cleanup");
            let end = context.labeler.unique_hint("end");

            let next = context.atom("next");
            let done = context.atom("done");

            context.scope.intermediate();
            context
                .write_instruction(Instruction::Uncons)
                .cond_jump(&setup)
                .write_label(enter.clone())
                .write_instruction(Instruction::Copy)
                .write_instruction(Instruction::Call(0))
                .write_instruction(Instruction::Copy)
                .write_instruction(Instruction::Const(done.into()))
                .write_instruction(Instruction::ValNeq)
                .cond_jump(&cleanup)
                .write_instruction(Instruction::Destruct)
                .write_instruction(Instruction::Swap)
                .write_instruction(Instruction::Const(next.into()))
                .write_instruction(Instruction::ValEq)
                .cond_jump(&cleanup);
            context.scope.intermediate();
            for pattern in &lookup.patterns {
                context.write_instruction(Instruction::Uncons);
                unbind(context, bound, pattern.bindings());
                write_pattern_match(context, pattern, &cleanup);
            }
            context.scope.end_intermediate();
            context
                .write_instruction(Instruction::Pop)
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(&end)
                .write_label(cleanup)
                .write_instruction(Instruction::Pop)
                .write_instruction(Instruction::Const(true.into()))
                .write_instruction(Instruction::Cons)
                .jump(on_fail);

            context.write_label(setup);
            // We can predetermine all the statically assigned expressions
            let mut set = HashSet::new();
            for (i, pattern) in lookup.patterns.iter().enumerate() {
                let vars = pattern.references();
                if vars.iter().all(|var| bound.is_bound(var)) {
                    set.insert(i.into());
                }
            }
            context.write_instruction(Instruction::Const(set.into()));
            let save_into = context.scope.intermediate();
            for (i, pattern) in lookup.patterns.iter().enumerate() {
                pattern.bindings();
                evaluate(context, bound, pattern, i, save_into);
                context.scope.intermediate();
            }

            context.write_instruction(Instruction::Call(lookup.patterns.len() + 1));
            for _ in &lookup.patterns {
                context.scope.end_intermediate();
            }
            context.scope.end_intermediate();
            context.jump(&enter).write_label(end);
        }
    }
}

fn unbind<'a>(context: &mut Context, bindset: Bindings<'_>, vars: HashSet<Id>) {
    let newly_bound = vars.difference(bindset.compile_time);
    for var in newly_bound {
        match context.scope.lookup(var).unwrap() {
            Binding::Variable(index) => {
                context.write_instruction(Instruction::UnsetLocal(index));
            }
            _ => unreachable!(),
        }
    }
}

fn evaluate(
    context: &mut Context,
    bindset: Bindings<'_>,
    value: &ir::Expression,
    expr_index: usize,
    save_into: usize,
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
                        .write_instruction(Instruction::LoadLocal(runtime_bound))
                        .write_instruction(Instruction::Const(index.into()))
                        .write_instruction(Instruction::Contains)
                        .cond_jump(&nope);
                }
                _ => unreachable!(),
            }
        }
        let next = context.labeler.unique_hint("next");
        write_expression(context, value);
        context
            .write_instruction(Instruction::LoadLocal(save_into))
            .write_instruction(Instruction::Const(expr_index.into()))
            .write_instruction(Instruction::Insert)
            .write_instruction(Instruction::Pop)
            .jump(&next)
            .write_label(nope)
            .write_instruction(Instruction::Const(().into()))
            .write_label(next);
    } else {
        context.write_instruction(Instruction::Const(().into()));
    }
}
