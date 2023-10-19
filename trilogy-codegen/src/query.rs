use crate::{preamble::ITERATE_COLLECTION, prelude::*};
use std::borrow::Cow;
use std::collections::HashSet;
use trilogy_ir::visitor::{HasBindings, HasReferences};
use trilogy_ir::{ir, Id};
use trilogy_vm::Instruction;

pub(crate) fn write_query_state(context: &mut Context, query: &ir::Query) {
    // Prepare the initial bindset and store it in a register. We'll be
    // referring to this register throughout.
    //
    // It is possible that as part of setting up a query state we need to
    // execute (including setup) a whole other query, so preserve the previous
    // register value.
    context.instruction(Instruction::LoadRegister(2));
    context.scope.intermediate();
    context
        // The initial bindset is empty for generic queries. Only for rules it's
        // a bit different, as the initial bindset is based on the boundness of the
        // parameters. Rule calls can just set that up and start from continue.
        .instruction(Instruction::Const(HashSet::new().into()))
        .instruction(Instruction::SetRegister(2));
    continue_query_state(context, query);
    // After writing out the query state, the final bindset doesn't matter (it's everything)
    // and the intermediate bindsets have been integrated with the state, so just reset the
    // register.
    context
        .instruction(Instruction::Swap)
        .instruction(Instruction::SetRegister(2));
    context.scope.end_intermediate();
}

// A continued query state is like a query state but the current bindset is currently on the
// top of the stack already.
//
// Outside of this file, other than for rule definitions, it is better to just
// call `write_query_state`, as there is no bindset. Inside this file, we use it for
// subqueries of queries.
pub(crate) fn write_continued_query_state(context: &mut Context, query: &ir::Query) {
    // The same saving of previous bindset, in case we are mid way setting up a query.
    context
        .instruction(Instruction::LoadRegister(2))
        .instruction(Instruction::Swap)
        .instruction(Instruction::SetRegister(2));
    context.scope.intermediate();
    continue_query_state(context, query);
    // Again the final continued bindset does not matter, just get rid of it and reset
    // to the previous one.
    context
        .instruction(Instruction::Swap)
        .instruction(Instruction::SetRegister(2));
    context.scope.end_intermediate();
}

// The actual writing of the query state happens here. The query state generally consists
// of two things:
// 1. Enough information that the query can be continued from where it left off
// 2. A bindset so that, during backtracking, we can unbind the variables that were bound
//    at each step.
//
// The delta to a bindset can be determined statically, and does not require backtracking,
// but the initial bindset cannot, in general, as it may depend on the arguments to a rule
// lookup. The only thing to be careful of is to store a clone of the bindset in any state
// that needs it, as otherwise it may get mutated in passing.
//
// "Continuing" the query state assumes that the current runtime bindset is in register 2.
//
fn continue_query_state(context: &mut Context, query: &ir::Query) {
    match &query.value {
        ir::QueryValue::Is(..) | ir::QueryValue::End | ir::QueryValue::Pass => {
            // These don't require much state and do not care about the bindset as they won't
            // backtrack, even internally. They will occur at most once, switching to false to
            // indicate that they should not occur again.
            context.instruction(Instruction::Const(true.into()));
        }
        ir::QueryValue::Not(..) => {
            // While a `not` query doesn't require much state (it's just an at most once query),
            // it actually can backtrack internally, so we have to keep the bindset. We also
            // have to reset the bindset back to the previous state once complete, since the
            // variables bound within a not query are not available outside.
            context
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone)
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons);
        }
        ir::QueryValue::Direct(unification) => {
            // A direct unification does affect the bindset, in passing, but only requires
            // the same "once-only" state as above.
            context.instruction(Instruction::Const(true.into()));

            context.instruction(Instruction::LoadRegister(2));
            for var in unification.pattern.bindings() {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context
                    .instruction(Instruction::Const(index.into()))
                    .instruction(Instruction::Insert);
            }
            context.instruction(Instruction::Pop);
        }
        ir::QueryValue::Element(unification) => {
            // Element unification iterates a collection. The state is an iterator
            // over this collection, which will eventually run out. Since this is
            // a potentially many times operation, we do need to store the bindset
            // as well.
            write_expression(context, &unification.expression);
            context
                .write_procedure_reference(ITERATE_COLLECTION.to_owned())
                .instruction(Instruction::Swap)
                .instruction(Instruction::Call(1))
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone)
                .instruction(Instruction::Cons);

            // Bindset updates happen *after* the state is computed because the backtracker
            // needs to be able to go back to the previous bindset.
            context.instruction(Instruction::LoadRegister(2));
            for var in unification.pattern.bindings() {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context
                    .instruction(Instruction::Const(index.into()))
                    .instruction(Instruction::Insert);
            }
            context.instruction(Instruction::Pop);
        }
        ir::QueryValue::Alternative(alt) => {
            // An alternative attempts the left side, and only if it fails attempts the right side.
            // Its state is the left side's state, and a marker to indicate that we've not yet
            // chosen a side. The `unit` becomes `true` or `false` later to indicate which side
            // has been chosen, once it has been chosen.
            //
            // Since there is no backtracking that happens at this node directly, the bindset
            // does not need to be kept.
            continue_query_state(context, &alt.0);
            context
                .instruction(Instruction::Const(().into()))
                .instruction(Instruction::Cons);
        }
        ir::QueryValue::Implication(alt) => {
            // Similarly, the state of the implication is its left side's state, and a marker
            // to indicate whether it was successfully matched or not. If it was not successfully
            // matched, the marker will remain false (under the assumption that it will fail to
            // match again if we come back in [might be a bad assumption?]). When it succeeds,
            // the marker becomes true.
            //
            // In either case, there's no backtracking done directly at this node, so no need to
            // keep a bindset.
            continue_query_state(context, &alt.0);
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons);
        }
        ir::QueryValue::Conjunction(alt) | ir::QueryValue::Disjunction(alt) => {
            // Conjuection and disjunction both work basically the same. The state is the
            // left state and a marker for whether we're on the first or second branch on
            // this iteration. The marker just changes on different conditions.
            //
            // Also, in both cases, the bindset must be kept because there will be backtracking.
            //
            // It's kind of just a coincidence that these can be represented with the same data.
            context
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone);
            context.scope.intermediate();
            continue_query_state(context, &alt.0);
            context
                .instruction(Instruction::Cons)
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons);
            context.scope.end_intermediate();
        }
        ir::QueryValue::Lookup(lookup) => {
            // Lookups require setting up the rule. All the values of the expression must be
            // bound ahead of time; there is no way to do something like `anything(x) and things(anything)`
            // it would have to go in reverse (`things(anything) and anything(x)`). This makes more
            // sense anyway. Works out because conjunctions and the like only prepare the state of their
            // first branch ahead of time, so by the time we need anything from a previous clause, it
            // will be bound already.
            //
            // Does require maintaining the bindset though.
            write_expression(context, &lookup.path);
            context
                .instruction(Instruction::Call(0))
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons);

            // After a lookup, all its patterns will be bound. This implies that we must iterate
            // every possible case for unbound patterns eagerly... but that's a sacrifice we have
            // to make. Doesn't really hurt correctness I think, but it is a bit less performant
            // than the optimal case, but... the optimal case gives SAT vibes so maybe it isn't
            // feasible anyway. (What does Prolog do?)
            context.instruction(Instruction::LoadRegister(2));
            for var in lookup
                .patterns
                .iter()
                .flat_map(|pat| pat.bindings())
                .collect::<HashSet<_>>()
            {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context
                    .instruction(Instruction::Const(index.into()))
                    .instruction(Instruction::Insert);
            }
            context.instruction(Instruction::Pop);
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
    write_query_value(
        context,
        &query.value,
        on_fail,
        &Bindings(Cow::Owned(HashSet::default())),
    );
}

/// Tracks statically which variables have been bound so far. Some variables may
/// additionally be bound due to runtime state, so this is not a complete view.
/// Runtime bindsets are tracked separately and used to properly un-bind variables
/// during the backtracking phase.
#[derive(Clone)]
pub(crate) struct Bindings<'a>(Cow<'a, HashSet<Id>>);

impl Bindings<'_> {
    fn is_bound(&self, var: &Id) -> bool {
        self.0.contains(var)
    }

    fn union(&self, other: &HashSet<Id>) -> Self {
        Self(Cow::Owned(self.0.union(other).cloned().collect()))
    }
}

impl<'a> AsRef<HashSet<Id>> for Bindings<'a> {
    fn as_ref(&self) -> &HashSet<Id> {
        &self.0
    }
}

fn write_query_value(
    context: &mut Context,
    value: &ir::QueryValue,
    on_fail: &str,
    bound: &Bindings<'_>,
) {
    match &value {
        ir::QueryValue::Direct(unification) => {
            // Set the state marker to false so we can't re-enter here.
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
            write_expression(context, &unification.expression);
            write_pattern_match(context, &unification.pattern, on_fail);
        }
        ir::QueryValue::Element(unification) => {
            let cleanup = context.labeler.unique_hint("in_cleanup");
            let continuation = context.labeler.unique_hint("in_cont");

            let next = context.atom("next");
            let done = context.atom("done");

            // Copy the state whole and work on a copy. Since it's an
            // iterator (closure) we don't need to update it manually
            // later, so just keep it in its place.
            context.instruction(Instruction::Copy);
            // Reset to the previous binding state
            context.instruction(Instruction::Uncons);
            unbind(context, bound, unification.pattern.bindings());
            // Then the query operates on the closure that is left.
            context
                // Done if done
                .instruction(Instruction::Call(0))
                .instruction(Instruction::Copy)
                .instruction(Instruction::Const(done.into()))
                .instruction(Instruction::ValNeq)
                .cond_jump(&cleanup)
                // Runtime type error is probably expected if it's not an iterator when an
                // iterator is expected, but we just go to fail instead anyway because it
                // seems easier for now.
                //
                // TODO: Maybe come back to that later, when a panic instruction is added?
                .instruction(Instruction::Copy)
                .instruction(Instruction::TypeOf)
                .instruction(Instruction::Const("struct".into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&cleanup) // Not a struct, so it is 'done
                .instruction(Instruction::Destruct)
                .instruction(Instruction::Const(next.into()))
                .instruction(Instruction::ValEq)
                .cond_jump(&cleanup); // Not a 'next, so invalid (fail as per comment above)

            // Success: bind the pattern to the value
            write_pattern_match(context, &unification.pattern, on_fail);

            // Then we're just done. The state is already fine. Lucky.
            context
                .jump(&continuation)
                .label(cleanup)
                .instruction(Instruction::Pop) // The 'done token
                .jump(on_fail)
                .label(continuation);
        }
        ir::QueryValue::Is(expr) => {
            // Set the state marker to false so we can't re-enter here.
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
            // Then it's just failed if the evaluation is false.
            write_expression(context, expr);
            context.cond_jump(on_fail);
        }
        ir::QueryValue::End => {
            // Always fail. The state isn't required at all.
            context.jump(on_fail);
        }
        ir::QueryValue::Pass => {
            // Always pass (the first time). We still don't re-enter
            // here, so it does "fail" the second time.
            context
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
        }
        ir::QueryValue::Not(query) => {
            let cleanup = context.labeler.unique_hint("not_cleanup");
            context
                // Take up the bindset for later
                .instruction(Instruction::Uncons)
                // Set the state marker to false so we can't re-enter here.
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Swap)
                .cond_jump(&cleanup);

            // The work here is to run the subquery as if it was a whole new query,
            // starting from scratch. We do need to pass along the current bindset
            // though, because a `not` subquery is treated as part of the parent
            // query in terms of scoping.
            let on_pass = context.labeler.unique_hint("not_fail");
            let bindset = context.scope.intermediate();
            context.scope.intermediate(); // Marker

            // The inner query state must continue from the current bindset,
            // without mutating it by accident.
            context
                .instruction(Instruction::LoadLocal(bindset))
                .instruction(Instruction::Clone);
            write_continued_query_state(context, query);
            // Then just immediately attempt the subquery.
            write_query_value(context, &query.value, &on_pass, bound);
            // If the subquery passes, it's actually supposed to be a fail.
            context
                .instruction(Instruction::Pop) // Discard the subquery state
                .label(&cleanup) // Then we leave via failure, fixing the `not` state back up
                .instruction(Instruction::Cons)
                .jump(on_fail);
            // Meanwhile, if the subquery fails, then that's a pass.
            context.label(on_pass).instruction(Instruction::Pop); // Again discard the internal state

            // First reset the bindset
            context.instruction(Instruction::LoadLocal(bindset));
            unbind(context, bound, query.value.bindings());
            // And finally fix the state
            context.instruction(Instruction::Cons);
            context.scope.end_intermediate();
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
            let rhs_bound = bound.union(&lhs_vars);

            context
                // The first field determines if we are on inner (true) or outer (false)
                .instruction(Instruction::Uncons)
                .cond_jump(&outer);

            // The inner query has its own state which got set up at the end of the outer
            // branch.
            //
            // Recommend reading the `outer` section first for context. The `inner` part
            // is executed in a continuation from there.
            context
                .label(inner.clone())
                // The state at this point is ((outer_bindset:outer_state):inner_state).
                // The outer state is just stored so we can reset with it when the inner
                // query finally fails, so we only need to carry it around.
                //
                // Pop the inner state off as it is all that is needed.
                .instruction(Instruction::Uncons);
            context.scope.intermediate(); // Outer state

            // There's nothing to reset to at this point, the inner query will do that
            // internally already, if needed.
            write_query_value(context, &conj.1.value, &reset, &rhs_bound);

            // On success, reconstruct the state
            context.scope.end_intermediate();
            context
                // Reattach the outer state
                .instruction(Instruction::Cons)
                // Put the marker on top
                .instruction(Instruction::Const(true.into()))
                .instruction(Instruction::Cons)
                .jump(&out);

            // On failure, simply reset with the outer query's state. It'll continue from
            // where it left off!
            context
                .label(reset)
                // Discard the inner state, it's garbage now.
                .instruction(Instruction::Pop)
                // The outer state is already on the stack.
                .instruction(Instruction::Reset);

            context
                .label(outer.clone())
                // Take out the bindset from the state. We will have to reset to this
                // one every time the outer query gets evaluated.
                .instruction(Instruction::Uncons);

            // Load up that bindset we just took out of the state and reset the bindings
            // accordingly.
            let outer_bindset = context.scope.intermediate();
            context.instruction(Instruction::LoadLocal(outer_bindset));
            unbind(context, bound, conj.0.value.bindings());
            // Then run the outer query as normal.
            write_query_value(context, &conj.0.value, &cleanup, bound);
            // When it succeeds, proceed to running the inner query.
            //
            // First, set up the inner query's state. This must continue from the current
            // bindset, which is the bindset from AFTER the outer query has been run. Add
            // all the outer query's bindings into the set now.
            context
                .instruction(Instruction::LoadLocal(outer_bindset))
                .instruction(Instruction::Clone);
            for var in conj.0.bindings() {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context
                    .instruction(Instruction::Const(index.into()))
                    .instruction(Instruction::Insert);
            }
            // Then build the state for the right hand query out of that.
            write_continued_query_state(context, &conj.1);

            // Next we do a bit of witchcraft with continuations.
            context
                // The state of the inner query is reconstructed as if it were fully brand new
                // and we weren't already halfway through the query.
                //
                // At this point the stack is outer_bindset, outer_state, inner_state.
                // We need to construct ((outer_bindset : outer_state):inner_state) so that things are easy later.
                .instruction(Instruction::Slide(2))
                .instruction(Instruction::Cons)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Cons)
                // Then we put that big tuple into register 3 briefly...
                .instruction(Instruction::SetRegister(3))
                // Create a continuation from here
                .shift(&next)
                // The continuation is the inner query.
                .jump(&inner);
            context
                .label(next)
                // After creating that continuation, immediately call it with the the temp
                // state as argument. This continuation will reset when the inner query finally
                // fails, at which point we continue as if it was outer being called.
                //
                // So long as the inner query succeeds, it will just... continue out into the
                // continuation, as it should.
                //
                // Tricky, but works.
                .instruction(Instruction::LoadRegister(3))
                .instruction(Instruction::Call(1))
                .jump(&outer);

            // We go here if the outer query fails. Once that happens, the whole conjunction is failed.
            context
                .label(cleanup)
                .instruction(Instruction::Cons) // Attach the state to the bindset
                .instruction(Instruction::Const(false.into()))
                .instruction(Instruction::Cons) // Then attach the marker
                .jump(on_fail);
            context.scope.end_intermediate(); // outer_bindset
            context.label(out);
        }
        ir::QueryValue::Implication(imp) => {
            let out = context.labeler.unique_hint("impl_out");
            let cleanup_first = context.labeler.unique_hint("impl_cleanf");
            let cleanup_second = context.labeler.unique_hint("impl_cleans");
            let outer = context.labeler.unique_hint("impl_outer");
            let inner = context.labeler.unique_hint("impl_inner");

            let lhs_vars = imp.0.bindings();
            let rhs_bound = bound.union(&lhs_vars);

            context.instruction(Instruction::Uncons).cond_jump(&outer);

            context.label(inner.clone());
            write_query_value(context, &imp.1.value, &cleanup_second, &rhs_bound);
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
            let enter = context.labeler.unique_hint("enter_lookup");
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
            for pattern in &lookup.patterns {
                pattern.bindings();
                evaluate(context, bound, pattern);
                context.scope.intermediate();
            }

            context.instruction(Instruction::Call(lookup.patterns.len() as u32));
            for _ in &lookup.patterns {
                context.scope.end_intermediate();
            }
            context.jump(&enter).label(end);
        }
    }
}

/// Unbinds the variables in `vars` that are runtime unbound but statically bound.
/// The runtime bindset is expected on top of stack, and will be consumed.
fn unbind(context: &mut Context, bindset: &Bindings<'_>, vars: HashSet<Id>) {
    let newly_bound = vars.difference(bindset.0.as_ref());
    for var in newly_bound {
        match context.scope.lookup(var).unwrap() {
            Binding::Variable(index) => {
                let skip = context.labeler.unique_hint("skip");
                context
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Const(index.into()))
                    .instruction(Instruction::Contains)
                    .instruction(Instruction::Not)
                    .cond_jump(&skip)
                    .instruction(Instruction::UnsetLocal(index))
                    .label(skip);
            }
            _ => unreachable!(),
        }
    }
    context.instruction(Instruction::Pop);
}

fn evaluate(context: &mut Context, bindset: &Bindings<'_>, value: &ir::Expression) {
    let vars = value.references();
    if vars.iter().all(|var| bindset.is_bound(var)) {
        // When all variables in this expression are statically determined to be bound,
        // no checking needs to be done
        write_expression(context, value);
    } else {
        // If some variables are not known statically, then those variables are checked
        // at runtime. If any are unset, then we just skip this expression and push an
        // empty cell on to the stack in its place.
        let nope = context.labeler.unique_hint("nope");
        for var in &vars {
            if bindset.is_bound(var) {
                continue;
            }
            match context.scope.lookup(var).unwrap() {
                Binding::Variable(index) => {
                    context
                        .instruction(Instruction::IsSetLocal(index))
                        .cond_jump(&nope);
                }
                _ => unreachable!(),
            }
        }
        let next = context.labeler.unique_hint("next");
        write_expression(context, value);
        context
            .jump(&next)
            .label(nope)
            .instruction(Instruction::Variable)
            .label(next);
    }
}
