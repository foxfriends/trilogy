use crate::INVALID_ITERATOR;
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
        .constant(HashSet::new())
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
            context.constant(true);
        }
        ir::QueryValue::Not(..) => {
            // While a `not` query doesn't require much state (it's just an at most once query),
            // it actually can backtrack internally, so we have to keep the bindset. We also
            // have to reset the bindset back to the previous state once complete, since the
            // variables bound within a not query are not available outside.
            context
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone)
                .constant(true)
                .instruction(Instruction::Cons);
        }
        ir::QueryValue::Direct(unification) => {
            // A direct unification does affect the bindset, in passing, but only requires
            // the same "once-only" state as above.
            context.constant(true);

            context.instruction(Instruction::LoadRegister(2));
            for var in unification.pattern.bindings() {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context.constant(index).instruction(Instruction::Insert);
            }
            context.instruction(Instruction::Pop);
        }
        ir::QueryValue::Element(unification) => {
            // Element unification iterates a collection. The state is an iterator
            // over this collection, which will eventually run out, but we can't evaluate
            // that yet because it might need some of its parameters bound still, so we
            // just put a unit as a placeholder.
            //
            // Since this is a potentially many times operation, we do need to store the
            // bindset as well.
            context
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone)
                .constant(())
                .instruction(Instruction::Cons);

            // Bindset updates happen *after* the state is computed because the backtracker
            // needs to be able to go back to the previous bindset.
            context.instruction(Instruction::LoadRegister(2));
            for var in unification.pattern.bindings() {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context.constant(index).instruction(Instruction::Insert);
            }
            context.instruction(Instruction::Pop);
        }
        ir::QueryValue::Alternative(alt) => {
            // An alternative attempts the left side, and only if it fails attempts the right side.
            //
            // Its state is the left side's state, and a marker to indicate that we've not yet
            // chosen a side. The `unit` becomes `true` or `false` later to indicate which side
            // has been chosen, once it has been chosen.
            //
            // The bindset is kept because there is a subquery which might need it, but not for any
            // backtracking.
            context
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone);
            context.scope.intermediate();
            continue_query_state(context, &alt.0);
            context
                .instruction(Instruction::Cons)
                .constant(())
                .instruction(Instruction::Cons);
            context.scope.end_intermediate();
        }
        ir::QueryValue::Implication(alt)
        | ir::QueryValue::Conjunction(alt)
        | ir::QueryValue::Disjunction(alt) => {
            // These all work basically the same. The state is the left state and a marker for whether
            // we're on the first or second branch on this iteration. The marker just changes on
            // different conditions.
            //
            // In all cases, the bindset must be kept because there is a subquery which will
            // need it for computing its own initial state.
            //
            // It's kind of just a coincidence that these can be represented with the same data.
            //
            // The structure is ((bindset:state):marker)
            context
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone);
            context.scope.intermediate();
            continue_query_state(context, &alt.0);
            context
                .instruction(Instruction::Cons)
                .constant(false)
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
            context
                .instruction(Instruction::LoadRegister(2))
                .instruction(Instruction::Clone);
            context.scope.intermediate();
            write_expression(context, &lookup.path);
            context
                .instruction(Instruction::Call(0)) // The state is the closure
                .instruction(Instruction::Cons) // with the bindset
                .constant(false) // and a flag to keep track of whether the closure is initialized or not
                .instruction(Instruction::Cons);
            context.scope.end_intermediate();

            // After a lookup, all its patterns will be bound. This implies that we must iterate
            // every possible case for unbound patterns eagerly... but that's a sacrifice we have
            // to make. Doesn't really hurt correctness I think, but it is a bit less performant
            // than the optimal case, but... the optimal case gives SAT vibes so maybe it isn't
            // even possible anyway. (What does Prolog do?)
            context.instruction(Instruction::LoadRegister(2));
            for var in lookup
                .patterns
                .iter()
                .flat_map(|pat| pat.bindings())
                .collect::<HashSet<_>>()
            {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context.constant(index).instruction(Instruction::Insert);
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
                .constant(false)
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
            context.scope.intermediate(); // state
            evaluate_or_fail(context, bound, &unification.expression, on_fail);
            write_pattern_match(context, &unification.pattern, on_fail);
            context.scope.end_intermediate(); // state
        }
        ir::QueryValue::Element(unification) => {
            let cleanup = context.labeler.unique_hint("in_cleanup");
            let continuation = context.labeler.unique_hint("in_cont");

            let body = context.labeler.unique_hint("in_body");

            // First check to see if the state is `unit`, meaning we have to set up the
            // iterator still.
            context.instruction(Instruction::Uncons);
            let bindset = context.scope.intermediate();
            context
                .instruction(Instruction::Copy)
                .constant(())
                .instruction(Instruction::ValEq)
                .cond_jump(&body)
                // State was unit, discard it
                .instruction(Instruction::Pop);
            // Then create the actual iterator
            let cleanup_setup = context.labeler.unique_hint("in_setup_fail");
            evaluate_or_fail(context, bound, &unification.expression, &cleanup_setup);
            context
                .write_procedure_reference(ITERATE_COLLECTION)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Call(1))
                .jump(&body)
                .label(&cleanup_setup)
                .constant(())
                .instruction(Instruction::Cons)
                .jump(on_fail);

            context.label(&body);
            context.scope.intermediate(); // state iterator

            // Reset to the previous binding state
            context.instruction(Instruction::LoadLocal(bindset));
            unbind(context, bound, unification.pattern.bindings());
            context
                .instruction(Instruction::Copy)
                .instruction(Instruction::Call(0))
                // Done if done
                .instruction(Instruction::Copy)
                .atom("done")
                .instruction(Instruction::ValNeq)
                .cond_jump(&cleanup)
                .instruction(Instruction::Copy)
                .instruction(Instruction::TypeOf)
                .atom("struct")
                .instruction(Instruction::ValEq)
                .cond_jump(INVALID_ITERATOR) // Not a struct, so it is runtime error
                .instruction(Instruction::Destruct)
                .atom("next")
                .instruction(Instruction::ValEq)
                .cond_jump(INVALID_ITERATOR);

            // Success: bind the pattern to the value. If this fails, move on to the next
            // element, not fail the whole query.
            write_pattern_match(context, &unification.pattern, &body);

            context
                .instruction(Instruction::Cons)
                .jump(&continuation)
                .label(cleanup)
                .instruction(Instruction::Pop) // The 'done token
                .instruction(Instruction::Cons)
                .jump(on_fail)
                .label(continuation);
            context.scope.end_intermediate(); // state iterator
            context.scope.end_intermediate(); // bindset
        }
        ir::QueryValue::Is(expr) => {
            // Set the state marker to false so we can't re-enter here.
            context
                .constant(false)
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
            context.scope.intermediate(); // state

            // Then it's just failed if the evaluation is false.
            write_expression(context, expr);
            context.scope.end_intermediate(); // state
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
                .constant(false)
                .instruction(Instruction::Swap)
                .cond_jump(on_fail);
        }
        ir::QueryValue::Not(query) => {
            let cleanup = context.labeler.unique_hint("not_cleanup");
            context
                // Take up the bindset for later
                .instruction(Instruction::Uncons)
                // Set the state marker to false so we can't re-enter here.
                .constant(false)
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
                .constant(true)
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
                context.constant(index).instruction(Instruction::Insert);
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
                .constant(false)
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

            // Deconstruct the state. The first field indicates whether we have already matched the
            // condition successfully (true) or not (false).
            context.instruction(Instruction::Uncons).cond_jump(&outer);

            // If the condition was successful, just run the inner query like normal.
            context.label(inner.clone());
            write_query_value(context, &imp.1.value, &cleanup_second, &rhs_bound);
            context
                // Only thing is to maintain the marker in the state.
                .constant(true)
                .instruction(Instruction::Cons)
                .jump(&out);

            // If the condition was not checked yet (or was checked and failed)
            // we have to run the condition.
            context
                .label(outer.clone())
                // Detach the state from the bindset
                .instruction(Instruction::Uncons);
            context.scope.intermediate(); // bindset

            // Don't need to unbind anything because we'll never backtrack if it succeeds.
            // Pretty simple after that, just run it.
            write_query_value(context, &imp.0.value, &cleanup_first, bound);
            // If it succeeds, discard the outer query's state, we don't need it
            // anymore because we'll never come back to it.
            context.instruction(Instruction::Pop);
            context.scope.end_intermediate(); // bindset
                                              // We're replacing it with the inner query's state. It does need the context
                                              // of the bindset though, which is conveniently on the stack already. Again, we
                                              // don't need to keep the bindset, so can just roll it up into the subquery.
                                              // We do need to add the bindings that were just set by the condition though.
            for var in imp.0.value.bindings() {
                let index = context.scope.lookup(&var).unwrap().unwrap_local();
                context.constant(index).instruction(Instruction::Insert);
            }
            write_continued_query_state(context, &imp.1);
            // Then move on to the inner query like nothing ever happened.
            context.jump(&inner);

            // If either query fails, we're just reconstructing the state so
            // that it is the same as before.
            context
                .label(cleanup_first)
                // The outer query has a bindset attached
                .instruction(Instruction::Cons)
                .constant(false)
                .instruction(Instruction::Cons)
                .jump(on_fail)
                .label(cleanup_second)
                // The inner query is opaque
                .constant(true)
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

            // Deconstruct the state. The first field indicates whether the first query has
            // already been exhausted (true) or not (false)
            context.instruction(Instruction::Uncons).cond_jump(&first);

            // If the first query is exhausted, we're just running the second one until it
            // is also exhausted.
            context.label(second.clone());
            write_query_value(context, &disj.1.value, &cleanup, bound);
            context
                // Only concern is to maintain the state
                .constant(true)
                .instruction(Instruction::Cons)
                .jump(&out);

            // Meanwhile if the first query is not exhausted, we're running it but do need
            // to worry about backtracking. This is much like conjunction.
            context.label(first).instruction(Instruction::Uncons);
            let bindset = context.scope.intermediate();
            context.instruction(Instruction::LoadLocal(bindset));
            unbind(context, bound, disj.0.value.bindings());

            write_query_value(context, &disj.0.value, &next, bound);
            // If it succeeds, just reconstruct the state before succeeding.
            context
                .instruction(Instruction::Cons)
                .constant(false)
                .instruction(Instruction::Cons)
                .jump(&out);

            // When it fails, instead of failing now, move on to the second branch.
            context.label(next).instruction(Instruction::Pop); // Discard the lhs state

            context.scope.end_intermediate();

            // The bindset is the same because the left hand side has not bound anything
            // at this point. Just pass it along, it's already on top of stack.
            write_continued_query_state(context, &disj.1);
            // Then jump to second and run like normal
            context.jump(&second);

            // Once the second one fails, then we're really failed.
            context
                .label(cleanup)
                .constant(true)
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

            // To run the alternative thing, we need to keep a little extra state
            // temporarily. Slip that in behind the actual state before we begin.
            context.constant(false).instruction(Instruction::Swap);
            let is_uncommitted = context.scope.intermediate();

            // First we determine which case we're on. One of three:
            // 1. committed first (true)
            // 2. committed second (false)
            // 3. uncommitted first (unit)
            context
                .instruction(Instruction::Uncons)
                // If it's unit, set the uncommitted flag
                .instruction(Instruction::Copy)
                .constant(())
                .instruction(Instruction::ValEq)
                .instruction(Instruction::SetLocal(is_uncommitted))
                // Then we do an equality check, making unit look the same as true so that
                // it runs the left side.
                .constant(false)
                .instruction(Instruction::ValNeq)
                .cond_jump(&second); // false = second! Backwards from other query types

            // If doing the left side, break out the bindset and reset to it so we can run the query.
            context.instruction(Instruction::Uncons);
            let bindset = context.scope.intermediate();
            context.instruction(Instruction::LoadLocal(bindset));
            unbind(context, bound, alt.0.value.bindings());
            write_query_value(context, &alt.0.value, &maybe, bound);
            // If it succeeds, fix the state and set the marker `true` so we come back here next time.
            context
                .instruction(Instruction::Cons)
                .constant(true)
                .instruction(Instruction::Cons)
                .jump(&out);
            context.scope.end_intermediate(); // bindset

            // If doing the right side, it's much the same as the left, but there is no bindset
            // because that will be handled by the subquery directly.
            context.label(second.clone());
            write_query_value(context, &alt.1.value, &cleanup_second, bound);
            context
                .constant(false)
                .instruction(Instruction::Cons)
                .jump(&out);

            // If the left side failed, we have to check if we were previously committed to the left
            // side or not.
            context
                .label(maybe)
                .instruction(Instruction::LoadLocal(is_uncommitted))
                // If we were committed (not uncommitted) then it's over, clean it all up.
                .cond_jump(&cleanup_first)
                // Otherwise, the left is failed but the right might not!
                // Discard the left's now useless state.
                .instruction(Instruction::Pop);
            // Then build up the right's state. The bindset is the same and conveniently already
            // on the top of stack.
            write_continued_query_state(context, &alt.1);
            // Then we can just run the second like normal.
            context.jump(&second);

            // When failing from the first side, reconstruct the state with its bindset
            context
                .label(cleanup_first)
                .instruction(Instruction::Cons)
                .constant(true)
                .instruction(Instruction::Cons)
                // Don't forget to discard the uncommitted flag
                .instruction(Instruction::Swap)
                .instruction(Instruction::Pop)
                .jump(on_fail);
            // Similar for the right side, but no bindset
            context
                .label(cleanup_second)
                .constant(false)
                .instruction(Instruction::Cons)
                // Don't forget to discard the uncommitted flag
                .instruction(Instruction::Swap)
                .instruction(Instruction::Pop)
                .jump(on_fail);

            // And finally, when successful we end up here, and still have to discard the uncommitted flag
            context
                .label(out)
                .instruction(Instruction::Swap)
                .instruction(Instruction::Pop);
            context.scope.end_intermediate(); // is_uncommitted
        }
        ir::QueryValue::Lookup(lookup) => {
            let setup = context.labeler.unique_hint("setup");
            let enter = context.labeler.unique_hint("enter_lookup");
            let cleanup = context.labeler.unique_hint("cleanup");
            let end = context.labeler.unique_hint("end");

            context
                .instruction(Instruction::Uncons)
                // The first field of the state is whether we've done set up already or not. If not,
                // then let's do the setup.
                .cond_jump(&setup);

            // If things are already ready to go, it's a matter of calling the query.
            context
                .label(enter.clone())
                // Begin by unbinding all the variables of all parameters, just because it's convenient to
                // do up front in one shot.
                .instruction(Instruction::Uncons);
            let bindset = context.scope.intermediate();
            context.instruction(Instruction::LoadLocal(bindset));
            unbind(
                context,
                bound,
                lookup
                    .patterns
                    .iter()
                    .flat_map(|pat| pat.bindings())
                    .collect(),
            );
            // Then we do the actual iteration of the lookup. Since it's just an iterator, we do the
            // iterator thing, pretty normally.
            context
                // Copy the iterator, it will just remain here for reconstructing the state
                .instruction(Instruction::Copy)
                // Then iterate the iterator
                .instruction(Instruction::Call(0))
                .instruction(Instruction::Copy)
                .atom("done")
                .instruction(Instruction::ValNeq)
                .cond_jump(&cleanup)
                .instruction(Instruction::Destruct)
                .atom("next")
                .instruction(Instruction::ValEq)
                .cond_jump(&cleanup);
            // If the iterator has yielded something, we have to destructure it into all the variables
            // of the unbound parameters.
            context.scope.intermediate(); // return value
            for pattern in &lookup.patterns {
                context.instruction(Instruction::Uncons);
                write_pattern_match(context, pattern, &cleanup);
            }
            context.scope.end_intermediate(); // return value
            context.scope.end_intermediate(); // bindset
            context
                // Discard the now empty yielded return value
                .instruction(Instruction::Pop)
                // Reconstruct the bindset:state
                .instruction(Instruction::Cons)
                // Put the marker back in too
                .constant(true)
                .instruction(Instruction::Cons)
                // And we're done!
                .jump(&end);

            // In the failure case, just reconstruct the state too
            context
                .label(cleanup)
                // Discard the invalid return value
                .instruction(Instruction::Pop)
                // Reattach the bindset:state
                .instruction(Instruction::Cons)
                // Put the marker back in too
                .constant(true)
                .instruction(Instruction::Cons)
                // And then call it failure
                .jump(on_fail);

            context.label(setup);
            // The setup of a lookup is to evaluate all the patterns and call the closure
            // that's currently in the state field, which will return an iterator that is
            // the actual query's state.
            //
            // First detach the bindset
            context.instruction(Instruction::Uncons);
            context.scope.intermediate(); // bindset
            context.scope.intermediate(); // rule closure

            // Then do the evaluation. Patterns with unbound variables will evaluate to
            // being unbound, as they are being used as output parameters at this time.
            for pattern in &lookup.patterns {
                evaluate(context, bound, pattern);
                context.scope.intermediate();
            }
            // Then we do the call, with all those arguments
            context.instruction(Instruction::Call(lookup.patterns.len() as u32));
            // Clean up the scope
            for _ in &lookup.patterns {
                context.scope.end_intermediate();
            }
            context.scope.end_intermediate(); // rule closure
            context.scope.end_intermediate(); // bindset
            context
                // Bindset gets reattached to scope
                .instruction(Instruction::Cons)
                // Then continue as if we didn't just set up
                .jump(&enter)
                .label(end);
        }
    }
}

/// Unbinds the variables in `vars` that are runtime unbound but statically bound.
/// The runtime bindset is expected on top of stack, and will be consumed.
fn unbind(context: &mut Context, bindset: &Bindings<'_>, vars: HashSet<Id>) {
    let newly_bound = vars.difference(bindset.0.as_ref());
    for var in newly_bound {
        let index = context.scope.lookup(var).unwrap().unwrap_local();
        let skip = context.labeler.unique_hint("skip");
        context
            .instruction(Instruction::Copy)
            .constant(index)
            .instruction(Instruction::Contains)
            .instruction(Instruction::Not)
            .cond_jump(&skip)
            .instruction(Instruction::UnsetLocal(index))
            .label(skip);
    }
    context.instruction(Instruction::Pop);
}

// Evaluate an expression for use in a lookup - in lookups the expressions might actually
// be patterns which are being un-evaluated later.
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

// Very similar to evaluate, but instead of writing an empty cell, just end. This is for
// use in unifications where some of the variables may not be bound due to them being
// used as output parameters for a rule.
fn evaluate_or_fail(
    context: &mut Context,
    bindset: &Bindings<'_>,
    value: &ir::Expression,
    on_fail: &str,
) {
    let vars = value.references();
    if vars.iter().all(|var| bindset.is_bound(var)) {
        // When all variables in this expression are statically determined to be bound,
        // no checking needs to be done
        write_expression(context, value);
    } else {
        // If some variables are not known statically, then those variables are checked
        // at runtime. If any are unset, then we just skip this expression and push an
        // empty cell on to the stack in its place.
        for var in &vars {
            if bindset.is_bound(var) {
                continue;
            }

            // If it's not a local variable, then it's definitely bound already because
            // it's static.
            if let Binding::Variable(index) = context.scope.lookup(var).unwrap() {
                context
                    .instruction(Instruction::IsSetLocal(index))
                    .cond_jump(on_fail);
            }
        }
        write_expression(context, value);
    }
}
