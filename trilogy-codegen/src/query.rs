use crate::evaluation::CodegenEvaluate;
use crate::{delegate_label_maker, delegate_stack_tracker};
use crate::{preamble::ITERATE_COLLECTION, prelude::*};
use std::borrow::Cow;
use std::collections::HashSet;
use trilogy_ir::visitor::{HasBindings, HasReferences, IrVisitable, IrVisitor};
use trilogy_ir::{ir, Id};
use trilogy_vm::{delegate_chunk_writer, Instruction};

struct QueryState<'b, 'a> {
    context: &'b mut Context<'a>,
}

pub(crate) trait CodegenQuery: IrVisitable {
    fn prepare_query(&self, context: &mut Context) {
        // Prepare the initial bindset and store it in a register. We'll be
        // referring to this register throughout.
        //
        // It is possible that as part of setting up a query state we need to
        // execute (including setup) a whole other query, so preserve the previous
        // register value.
        context
            .instruction(Instruction::LoadRegister(BINDSET))
            .intermediate();
        context
            // The initial bindset is empty for generic queries. Only for rules it's
            // a bit different, as the initial bindset is based on the boundness of the
            // parameters. Rule calls can just set that up and start from continue.
            .constant(HashSet::new())
            .instruction(Instruction::SetRegister(BINDSET));
        self.visit(&mut QueryState { context });
        // After writing out the query state, the final bindset doesn't matter (it's everything)
        // and the intermediate bindsets have been integrated with the state, so just reset the
        // register.
        context
            .instruction(Instruction::Swap)
            .instruction(Instruction::SetRegister(BINDSET))
            .end_intermediate();
    }

    fn extend_query_state(&self, context: &mut Context) {
        // The same saving of previous bindset, in case we are mid way setting up a query.
        context
            .instruction(Instruction::LoadRegister(BINDSET))
            .instruction(Instruction::Swap)
            .instruction(Instruction::SetRegister(BINDSET))
            .intermediate();
        self.visit(&mut QueryState { context });
        // Again the final continued bindset does not matter, just get rid of it and reset
        // to the previous one.
        context
            .instruction(Instruction::Swap)
            .instruction(Instruction::SetRegister(BINDSET))
            .end_intermediate();
    }

    fn execute_query(&self, context: &mut Context, on_fail: &str) {
        self.visit(&mut QueryEvaluation {
            context,
            on_fail,
            bindings: &Bindings::default(),
        });
    }
}

impl CodegenQuery for ir::Query {}
impl CodegenQuery for ir::QueryValue {}

impl QueryState<'_, '_> {
    fn add_bindings(&mut self, vars: impl IntoIterator<Item = Id>) {
        for var in vars {
            let index = self.context.scope.lookup(&var).unwrap().unwrap_local();
            self.context
                .constant(index)
                .instruction(Instruction::Insert);
        }
        self.context.instruction(Instruction::Pop);
    }

    fn junction(&mut self, node: &(ir::Query, ir::Query)) {
        self.context
            .instruction(Instruction::LoadRegister(BINDSET))
            .instruction(Instruction::Clone)
            .intermediate();
        node.0.visit(self);
        self.context
            .instruction(Instruction::Cons)
            .constant(false)
            .instruction(Instruction::Cons)
            .end_intermediate();
    }
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
impl IrVisitor for QueryState<'_, '_> {
    // These don't require much state and do not care about the bindset as they won't
    // backtrack, or spawn children. They will occur at most once, switching to false to
    // indicate that they should not occur again.

    fn visit_query_is(&mut self, _: &ir::Expression) {
        self.context.constant(true);
    }

    fn visit_query_pass(&mut self) {
        self.context.constant(true);
    }

    fn visit_query_fail(&mut self) {
        self.context.constant(false);
    }

    // While a `not` query doesn't require much state (it's just an at most once query),
    // it actually can backtrack internally, so we have to keep the bindset. We also
    // have to reset the bindset back to the previous state once complete, since the
    // variables bound within a not query are not available outside.
    fn visit_query_not(&mut self, _: &ir::Query) {
        self.context
            .instruction(Instruction::LoadRegister(BINDSET))
            .instruction(Instruction::Clone)
            .constant(true)
            .instruction(Instruction::Cons);
    }

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

    fn visit_query_conjunction(&mut self, node: &(ir::Query, ir::Query)) {
        self.junction(node);
    }

    fn visit_query_disjunction(&mut self, node: &(ir::Query, ir::Query)) {
        self.junction(node);
    }

    fn visit_query_implication(&mut self, node: &(ir::Query, ir::Query)) {
        self.junction(node);
    }

    // An alternative attempts the left side, and only if it fails attempts the right side.
    //
    // Its state is the left side's state, and a marker to indicate that we've not yet
    // chosen a side. The `unit` becomes `true` or `false` later to indicate which side
    // has been chosen, once it has been chosen.
    //
    // The bindset is kept because there is a subquery which might need it, but not for any
    // backtracking.
    fn visit_query_alternative(&mut self, node: &(ir::Query, ir::Query)) {
        self.context
            .instruction(Instruction::LoadRegister(BINDSET))
            .instruction(Instruction::Clone)
            .intermediate();
        node.0.visit(self);
        self.context
            .instruction(Instruction::Cons)
            .constant(())
            .instruction(Instruction::Cons)
            .end_intermediate();
    }

    // The direct unification happens at most once, and does produce bindings so it
    // carries its own bindset.
    fn visit_direct_unification(&mut self, node: &ir::Unification) {
        self.context
            .instruction(Instruction::LoadRegister(BINDSET))
            .instruction(Instruction::Clone)
            .constant(true)
            .instruction(Instruction::Cons)
            .instruction(Instruction::LoadRegister(BINDSET));
        self.add_bindings(node.pattern.bindings());
    }

    // Element unification iterates a collection. The state is a continuation to the
    // iteration in progress. When the iteration has not yet begun, it's just the bindset.
    fn visit_element_unification(&mut self, node: &ir::Unification) {
        self.context
            .instruction(Instruction::LoadRegister(BINDSET))
            .instruction(Instruction::Clone)
            // Bindset updates happen *after* the state is computed because the backtracker
            // needs to be able to go back to the previous bindset.
            .instruction(Instruction::LoadRegister(BINDSET));
        self.add_bindings(node.pattern.bindings());
    }

    fn visit_lookup(&mut self, node: &ir::Lookup) {
        // Lookups require setting up the rule. All the values of the expression must be
        // bound ahead of time; there is no way to do something like `anything(x) and things(anything)`
        // it would have to go in reverse (`things(anything) and anything(x)`). This makes more
        // sense anyway. Works out because conjunctions and the like only prepare the state of their
        // first branch ahead of time, so by the time we need anything from a previous clause, it
        // will be bound already.
        //
        // Does require maintaining the bindset though.
        self.context
            .instruction(Instruction::LoadRegister(BINDSET))
            .instruction(Instruction::Clone)
            .intermediate();
        self.context
            .evaluate(&node.path)
            .typecheck("callable")
            .raw_call(0)
            .instruction(Instruction::Cons) // with the bindset
            .constant(false) // and a flag to keep track of whether the closure is initialized or not
            .instruction(Instruction::Cons)
            .end_intermediate();

        // After a lookup, all its patterns will be bound. This implies that we must iterate
        // every possible case for unbound patterns eagerly... but that's a sacrifice we have
        // to make. Doesn't really hurt correctness I think, but it is a bit less performant
        // than the optimal case, but... the optimal case gives SAT vibes so maybe it isn't
        // even possible anyway. (What does Prolog do?)
        self.context.instruction(Instruction::LoadRegister(BINDSET));
        self.add_bindings(
            node.patterns
                .iter()
                .flat_map(|pat| pat.bindings())
                .collect::<HashSet<_>>(),
        );
    }
}

/// Tracks statically which variables have been bound so far. Some variables may
/// additionally be bound due to runtime state, so this is not a complete view.
/// Runtime bindsets are tracked separately and used to properly un-bind variables
/// during the backtracking phase.
#[derive(Clone, Default)]
struct Bindings<'a>(Cow<'a, HashSet<Id>>);

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

struct QueryEvaluation<'b, 'a> {
    context: &'b mut Context<'a>,
    bindings: &'b Bindings<'a>,
    on_fail: &'b str,
}

delegate_chunk_writer!(QueryEvaluation<'_, '_>, context);
delegate_stack_tracker!(QueryEvaluation<'_, '_>, context);
delegate_label_maker!(QueryEvaluation<'_, '_>, context);

impl<'b, 'a> QueryEvaluation<'b, 'a> {
    fn fail(&mut self) -> &mut Self {
        self.jump(self.on_fail)
    }

    fn cond_fail(&mut self) -> &mut Self {
        self.cond_jump(self.on_fail)
    }

    /// Unbinds the variables in `vars` that are runtime unbound but statically bound.
    /// The runtime bindset is expected on top of stack, and will be consumed.
    fn unbind(&mut self, vars: HashSet<Id>) -> &mut Self {
        let newly_bound = vars.difference(self.bindings.0.as_ref());
        for var in newly_bound {
            let index = self.context.scope.lookup(var).unwrap().unwrap_local();
            let skip = self.make_label("skip");
            self.instruction(Instruction::Copy)
                .constant(index)
                .instruction(Instruction::Contains)
                .instruction(Instruction::Not)
                .cond_jump(&skip)
                .instruction(Instruction::UnsetLocal(index))
                .label(skip);
        }
        self.instruction(Instruction::Pop)
    }

    // Evaluate an expression for use in a lookup - in lookups the expressions might actually
    // be patterns which are being un-evaluated later.
    fn evaluate_or_skip<E: CodegenEvaluate>(&mut self, value: &E) -> &mut Self {
        let vars = value.references();
        if vars.iter().all(|var| self.bindings.is_bound(var)) {
            // When all variables in this expression are statically determined to be bound,
            // no checking needs to be done
            self.context.evaluate(value);
        } else {
            // If some variables are not known statically, then those variables are checked
            // at runtime. If any are unset, then we just skip this expression and push an
            // empty cell on to the stack in its place.
            let nope = self.make_label("nope");
            for var in &vars {
                if self.bindings.is_bound(var) {
                    continue;
                }
                // If it was static or context, it's definitely already bound. Only variables
                // might be unset yet.
                if let Binding::Variable(index) = self.context.scope.lookup(var).unwrap() {
                    self.instruction(Instruction::IsSetLocal(index))
                        .cond_jump(&nope);
                }
            }
            self.context.evaluate(value).bubble(|c| {
                c.label(nope).instruction(Instruction::Variable);
            });
        }
        self
    }

    // Very similar to evaluate, but instead of writing an empty cell, just end. This is for
    // use in unifications where some of the variables may not be bound due to them being
    // used as output parameters for a rule.
    fn evaluate_or_fail<E: CodegenEvaluate>(&mut self, value: &E, on_fail: &str) -> &mut Self {
        let vars = value.references();
        if vars.iter().all(|var| self.bindings.is_bound(var)) {
            // When all variables in this expression are statically determined to be bound,
            // no checking needs to be done
            self.context.evaluate(value);
        } else {
            // If some variables are not known statically, then those variables are checked
            // at runtime. If any are unset, then we just skip this expression and push an
            // empty cell on to the stack in its place.
            for var in &vars {
                if self.bindings.is_bound(var) {
                    continue;
                }

                // If it's not a local variable, then it's definitely bound already because
                // it's static.
                if let Binding::Variable(index) = self.context.scope.lookup(var).unwrap() {
                    self.instruction(Instruction::IsSetLocal(index))
                        .cond_jump(on_fail);
                }
            }
            self.context.evaluate(value);
        }
        self
    }

    fn execute_subquery<E: CodegenQuery>(
        &mut self,
        value: &E,
        on_fail: &str,
        bindings: &Bindings<'a>,
    ) -> &mut Self {
        value.visit(&mut QueryEvaluation {
            context: self.context,
            on_fail,
            bindings,
        });
        self
    }
}

// Query expects the top of the stack to be the query's current state.
//
// Upon finding some solution, the bindings have been set, and the new state is left on
// the top of the stack.
//
// On failure to find another solution, jump to `on_fail` with the state value left so
// that all further requests will also fail.
//
// It's up to the user of the query to call `write_query_state` to ensure that the
// initial state is set, as well as to ensure that the state value is carried around
// so that next time we enter the query it is set correctly.
impl IrVisitor for QueryEvaluation<'_, '_> {
    fn visit_direct_unification(&mut self, unification: &ir::Unification) {
        let cleanup = self.make_label("cleanup");
        // Take out the bindset
        let bindset = self.instruction(Instruction::Uncons).intermediate();
        // Set the state marker to false so we can't re-enter here.
        self.constant(false)
            .instruction(Instruction::Swap)
            .cond_jump(&cleanup)
            .intermediate(); // state marker
        self.evaluate_or_fail(&unification.expression, self.on_fail);
        self.intermediate(); // value
        self.instruction(Instruction::LoadLocal(bindset));
        // Unbind the bindset only right before assignment
        self.unbind(unification.pattern.bindings());
        self.end_intermediate(); // value
        self.context.pattern_match(&unification.pattern, &cleanup);
        self.end_intermediate() // state marker
            .instruction(Instruction::Cons)
            .bubble(|c| {
                c.label(cleanup).instruction(Instruction::Cons).fail();
            })
            .end_intermediate(); // bindset
    }

    fn visit_element_unification(&mut self, unification: &ir::Unification) {
        let begin = self.make_label("in_begin");
        let end = self.make_label("in_end");
        let cleanup = self.make_label("in_cleanup");
        let skip = self.make_label("in_skip");

        let query_state = self.intermediate();
        // If the state is a callable, that means we're mid-iteration already, continue
        // by calling it, provided with continuations onto which to return.
        self.try_type("callable", Err(&begin))
            .continuation(|context| {
                context
                    // NOTE: continuation has 2 extra items on the stack which must be
                    // discarded before jumping out.
                    //
                    // One extra for the previous query state
                    .instruction(Instruction::Slide(3))
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop)
                    .fail();
            })
            .intermediate();
        self.continuation(|context| {
            context
                // NOTE: continuation has 2 extra items on the stack which must be
                // discarded before jumping out.
                //
                // Two extra for the previous query state, and fail continuation
                .instruction(Instruction::Slide(4))
                .instruction(Instruction::Pop)
                .instruction(Instruction::Pop)
                .instruction(Instruction::Pop)
                .instruction(Instruction::Pop)
                .jump(&end);
        })
        .end_intermediate()
        .instruction(Instruction::Become(2));

        // Alternatively, begin the iteration:
        //
        // Create a continuation onto which to continue for a fail
        let fail_continuation = self
            .label(&begin)
            .continuation(|context| {
                context
                    // NOTE: continuation has 2 extra items on the stack which must be
                    // discarded before jumping out.
                    //
                    // One extra for the previous query state
                    .instruction(Instruction::Slide(3))
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop)
                    .fail();
            })
            .intermediate();
        // Create a continuation onto which to continue for a pass
        let pass_continuation = self
            .continuation(|context| {
                context
                    // NOTE: continuation has 2 extra items on the stack which must be
                    // discarded before jumping out.
                    //
                    // Two extra for the previous query state, and fail continuation
                    .instruction(Instruction::Slide(4))
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop)
                    .instruction(Instruction::Pop)
                    .jump(&end);
            })
            .intermediate();
        self.iterate(
            |context, params| {
                // For each element yielded, the query binds the value to the pattern, then
                // "returns" the state of the query, which is a continuation that resumes
                // from here.
                //
                // If the pattern fails to bind, skip this iteration but move on to the next,
                // don't fail the query.
                context.context.pattern_match(&unification.pattern, &skip);
                // A pass is performed by calling the "pass" continuation with the
                // query state (which is in turn a continuation back to here).
                context
                    .instruction(Instruction::LoadLocal(pass_continuation))
                    .intermediate();
                context
                    .continuation(|context| {
                        // This "resume" is called with the new pass continuation
                        context
                            .instruction(Instruction::SetLocal(pass_continuation))
                            .instruction(Instruction::SetLocal(fail_continuation))
                            // NOTE: continuation has 2 extra items on the stack which must be
                            // discarded before jumping out.
                            .instruction(Instruction::Pop)
                            .instruction(Instruction::Pop)
                            // When continuing the iterator, start by unbinding the pattern, then
                            // just resume and cancel the iterator as in standard iteration.
                            .label(&skip)
                            .instruction(Instruction::LoadLocal(query_state))
                            .unbind(unification.pattern.bindings())
                            .instruction(Instruction::LoadLocal(params.cancel))
                            .instruction(Instruction::LoadLocal(params.resume))
                            // Resume the iterator with unit, since the value won't be used
                            .constant(())
                            .call_function()
                            // The iterator eventually evaluates to something, which gets passed up
                            .become_function();
                    })
                    .end_intermediate()
                    .instruction(Instruction::Become(1));
            },
            |context| {
                // Since we allow this to either be an iterator or a collection, after evaluating
                // we attempt to iterate it as a collection. An iterator used as a value should evaluate
                // to unit, and unit is the same as empty list, so it gracefully handles the iterator
                // case by just iterating while evaluating.
                //
                // If the value is not an iterator or collection, then ITERATE_COLLECTION will panic.
                //
                // NOTE: The weird case is if you write a procedural iterator that then returns some
                // array, and then this iterates that array right after... like a double iteration.
                // TODO: Maybe fix that later, if it confuses people.
                context
                    .evaluate_or_fail(&unification.expression, &cleanup)
                    .reference(ITERATE_COLLECTION)
                    .instruction(Instruction::Swap)
                    // ITERATE_COLLECTION is not a real function, so the call is bare.
                    .instruction(Instruction::Call(1))
                    // Afterwards, we just end, as there are no more items to iterate.
                    .label(&cleanup);
            },
        )
        // Once iteration is complete, the query finally fails
        .instruction(Instruction::Pop) // Final "value" of iteration
        .instruction(Instruction::Pop) // Pass continuation
        .end_intermediate() // pass continuation
        .instruction(Instruction::Swap) // Swap bindset with fail continuation to call it
        .end_intermediate() // bindset is no longer intermediate
        .end_intermediate() // neither is fail continuation
        .instruction(Instruction::Become(1))
        .label(&end);
    }

    fn visit_query_is(&mut self, expr: &ir::Expression) {
        // Set the state marker to false so we can't re-enter here.
        self.constant(false)
            .instruction(Instruction::Swap)
            .cond_fail()
            .intermediate(); // state

        // Then it's just failed if the evaluation is false.
        self.context.evaluate(expr);
        self.end_intermediate() // state
            .typecheck("boolean")
            .cond_fail();
    }

    // Always fail. The state isn't required at all.
    fn visit_query_fail(&mut self) {
        self.fail();
    }

    fn visit_query_pass(&mut self) {
        // Always pass (the first time). We still don't re-enter
        // here, so it does "fail" the second time.
        self.constant(false)
            .instruction(Instruction::Swap)
            .cond_fail();
    }

    fn visit_query_not(&mut self, query: &ir::Query) {
        let cleanup = self.make_label("not_cleanup");
        self
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
        let on_pass = self.make_label("not_fail");
        let bindset = self.intermediate();
        self.intermediate(); // Marker

        // The inner query state must continue from the current bindset,
        // without mutating it by accident.
        self.instruction(Instruction::LoadLocal(bindset))
            .instruction(Instruction::Clone);
        self.context.extend_query_state(query);
        // Then just immediately attempt the subquery.
        self.execute_subquery(&query.value, &on_pass, self.bindings)
            // If the subquery passes, it's actually supposed to be a fail.
            .instruction(Instruction::Pop) // Discard the subquery state
            .label(&cleanup) // Then we leave via failure, fixing the `not` state back up
            .instruction(Instruction::Cons)
            .fail();
        // Meanwhile, if the subquery fails, then that's a pass.
        self.label(on_pass).instruction(Instruction::Pop); // Again discard the internal state

        // Then reset the bindset
        self.instruction(Instruction::LoadLocal(bindset));
        self.unbind(query.value.bindings())
            // And finally fix the state
            .instruction(Instruction::Cons)
            .end_intermediate()
            .end_intermediate();
    }

    fn visit_query_conjunction(&mut self, conj: &(ir::Query, ir::Query)) {
        let cleanup = self.make_label("conj_cleanup");
        let outer = self.make_label("conj_outer");
        let inner = self.make_label("conj_inner");
        let reset = self.make_label("conj_reset");

        let lhs_vars = conj.0.bindings();
        let rhs_bound = self.bindings.union(&lhs_vars);

        self
            // The first field determines if we are on inner (true) or outer (false)
            .instruction(Instruction::Uncons)
            .cond_jump(&outer)
            // The inner query has its own state which got set up at the end of the outer
            // branch.
            //
            // Recommend reading the `outer` section first for context.
            .label(inner.clone())
            // The state at this point is ((outer_bindset:outer_state):inner_state).
            // The outer state is stored so that we can continue the outer query when
            // the inner one finally fails.
            //
            // Pop the inner state off as it is all that is needed.
            .instruction(Instruction::Uncons)
            .intermediate(); // Outer state

        // There's nothing to reset to at this point, the inner query will do that
        // internally already, if needed.
        self.execute_subquery(&conj.1.value, &reset, &rhs_bound)
            // On success, reconstruct the state
            .end_intermediate()
            // Reattach the outer state
            .instruction(Instruction::Cons)
            // Put the marker on top
            .constant(true)
            .instruction(Instruction::Cons)
            .bubble(|context| {
                // On failure, simply reset with the outer query's state. It'll continue from
                // where it left off!
                context
                    .label(reset)
                    // Discard the inner state, it's garbage now.
                    .instruction(Instruction::Pop)
                    // The outer state is already on the stack.
                    .label(outer)
                    // Take out the bindset from the state. We will have to reset to this
                    // one every time the outer query gets evaluated.
                    .instruction(Instruction::Uncons);

                // Load up that bindset we just took out of the state and reset the bindings
                // accordingly.
                let outer_bindset = context.intermediate();
                context
                    .instruction(Instruction::LoadLocal(outer_bindset))
                    .unbind(conj.0.value.bindings());
                // Then run the outer query as normal.
                context
                    .execute_subquery(&conj.0.value, &cleanup, context.bindings)
                    // When it succeeds, proceed to running the inner query.
                    //
                    // First, set up the inner query's state. This must continue from the current
                    // bindset, which is the bindset from AFTER the outer query has been run. Add
                    // all the outer query's bindings into the set now.
                    .instruction(Instruction::LoadLocal(outer_bindset))
                    .instruction(Instruction::Clone);
                for var in conj.0.bindings() {
                    let index = context.context.scope.lookup(&var).unwrap().unwrap_local();
                    context.constant(index).instruction(Instruction::Insert);
                }
                // Then build the state for the right hand query out of that.
                context
                    .context
                    .extend_query_state(&conj.1)
                    // The state of the inner query is reconstructed as if it were fully brand new
                    // and we weren't already halfway through the query.
                    //
                    // At this point the stack is outer_bindset, outer_state, inner_state.
                    // We need to construct ((outer_bindset : outer_state):inner_state) so that things are easy later.
                    .instruction(Instruction::Slide(2))
                    .instruction(Instruction::Cons)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Cons)
                    .jump(inner);

                // We go here if the outer query fails. Once that happens, the whole conjunction is failed.
                context
                    .label(cleanup)
                    .instruction(Instruction::Cons) // Attach the state to the bindset
                    .constant(false)
                    .instruction(Instruction::Cons) // Then attach the marker
                    .fail()
                    .end_intermediate(); // outer_bindset
            });
    }

    fn visit_query_implication(&mut self, imp: &(ir::Query, ir::Query)) {
        let cleanup_first = self.make_label("impl_cleanf");
        let cleanup_second = self.make_label("impl_cleans");
        let outer = self.make_label("impl_outer");
        let inner = self.make_label("impl_inner");

        let lhs_vars = imp.0.bindings();
        let rhs_bound = self.bindings.union(&lhs_vars);

        // Deconstruct the state. The first field indicates whether we have already matched the
        // condition successfully (true) or not (false).
        self.instruction(Instruction::Uncons).cond_jump(&outer);

        // If the condition was previously successful, just run the inner query like normal.
        self.label(inner.clone());
        self.execute_subquery(&imp.1.value, &cleanup_second, &rhs_bound)
            // Only thing is to maintain the marker in the state.
            .constant(true)
            .instruction(Instruction::Cons);

        self.bubble(|context| {
            // If the condition was not checked yet (or was checked and failed)
            // we have to run the condition.
            context
                .label(outer.clone())
                // Detach the state from the bindset
                .instruction(Instruction::Uncons)
                .intermediate(); // bindset

            // Don't need to unbind anything because we'll never backtrack if it succeeds.
            // Pretty simple after that, just run it.
            context
                .execute_subquery(&imp.0.value, &cleanup_first, context.bindings)
                // If it succeeds, discard the outer query's state, we don't need it
                // anymore because we'll never come back to it.
                .instruction(Instruction::Pop)
                .end_intermediate(); // bindset

            // We're replacing it with the inner query's state. It does need the context
            // of the bindset though, which is conveniently on the stack already. Again, we
            // don't need to keep the bindset, so can just roll it up into the subquery.
            // We do need to add the bindings that were just set by the condition though.
            for var in imp.0.value.bindings() {
                let index = context.context.scope.lookup(&var).unwrap().unwrap_local();
                context.constant(index).instruction(Instruction::Insert);
            }
            context.context.extend_query_state(&imp.1);
            context
                // Then move on to the inner query like nothing ever happened.
                .jump(&inner)
                // If either query fails, we're just reconstructing the state so
                // that it is the same as before.
                .label(cleanup_first)
                // The outer query has a bindset attached
                .instruction(Instruction::Cons)
                .constant(false)
                .instruction(Instruction::Cons)
                .fail()
                .label(cleanup_second)
                // The inner query is opaque
                .constant(true)
                .instruction(Instruction::Cons)
                .fail();
        });
    }

    fn visit_query_disjunction(&mut self, disj: &(ir::Query, ir::Query)) {
        let first = self.make_label("disj_first");
        let second = self.make_label("disj_second");
        let next = self.make_label("disj_next");
        let cleanup = self.make_label("disj_cleanup");

        // Deconstruct the state. The first field indicates whether the first query has
        // already been exhausted (true) or not (false)
        self.instruction(Instruction::Uncons)
            .cond_jump(&first)
            // If the first query is exhausted, we're just running the second one until it
            // is also exhausted.
            .label(second.clone());
        self.execute_subquery(&disj.1.value, &cleanup, self.bindings)
            // Only concern is to maintain the state
            .constant(true)
            .instruction(Instruction::Cons);

        self.bubble(|context| {
            // Meanwhile if the first query is not exhausted, we're running it but do need
            // to worry about backtracking. This is much like conjunction.
            let bindset = context
                .label(first)
                .instruction(Instruction::Uncons)
                .intermediate();
            context
                .instruction(Instruction::LoadLocal(bindset))
                .unbind(disj.0.value.bindings());
            context
                .execute_subquery(&disj.0.value, &next, context.bindings)
                // If it succeeds, just reconstruct the state before succeeding.
                .instruction(Instruction::Cons)
                .constant(false)
                .instruction(Instruction::Cons)
                .bubble(|context| {
                    // When it fails, instead of failing now, move on to the second branch.
                    context
                        .label(next)
                        .instruction(Instruction::Pop) // Discard the lhs state
                        .end_intermediate();
                    // The bindset is the same because the left hand side has not self.bindings anything
                    // at this point. Just pass it along, it's already on top of stack.
                    context.context.extend_query_state(&disj.1);
                    // Then jump to second and run like normal
                    context
                        .jump(&second)
                        // Once the second one fails, then we're really failed.
                        .label(cleanup)
                        .constant(true)
                        .instruction(Instruction::Cons)
                        .fail();
                });
        });
    }

    fn visit_query_alternative(&mut self, alt: &(ir::Query, ir::Query)) {
        let maybe = self.make_label("alt_maybe");
        let second = self.make_label("alt_second");
        let cleanup_first = self.make_label("alt_cleanf");
        let cleanup_second = self.make_label("alt_cleans");

        // To run the alternative thing, we need to keep a little extra state
        // temporarily. Slip that in behind the actual state before we begin.
        let is_uncommitted = self
            .constant(false)
            .instruction(Instruction::Swap)
            .intermediate();

        // First we determine which case we're on. One of three:
        // 1. committed first (true)
        // 2. committed second (false)
        // 3. uncommitted first (unit)
        self.instruction(Instruction::Uncons)
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

        // If doing the left side, break out the bindset, we won't be needing it.
        self.instruction(Instruction::Uncons).intermediate();
        self.execute_subquery(&alt.0.value, &maybe, self.bindings)
            // If it succeeds, fix the state and set the marker `true` so we come back here next time.
            .instruction(Instruction::Cons)
            .constant(true)
            .instruction(Instruction::Cons)
            .bubble(|context| {
                // If doing the right side, it's much the same as the left, but there is no bindset
                // because that will be handled by the subquery directly.
                context
                    .end_intermediate() // bindset // TODO: is this in the right spot?
                    .label(second.clone());
                context
                    .execute_subquery(&alt.1.value, &cleanup_second, context.bindings)
                    .constant(false)
                    .instruction(Instruction::Cons)
                    .bubble(|context| {
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
                        context.context.extend_query_state(&alt.1);
                        // Then we can just run the second like normal.
                        context
                            .jump(&second)
                            // When failing from the first side, reconstruct the state with its bindset
                            .label(cleanup_first)
                            .instruction(Instruction::Cons)
                            .constant(true)
                            .instruction(Instruction::Cons)
                            // Don't forget to discard the uncommitted flag
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Pop)
                            .fail()
                            // Similar for the right side, but no bindset
                            .label(cleanup_second)
                            .constant(false)
                            .instruction(Instruction::Cons)
                            // Don't forget to discard the uncommitted flag
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Pop)
                            .fail();
                    });
            })
            // And finally, when successful we end up here, and still have to discard the uncommitted flag
            .instruction(Instruction::Swap)
            .instruction(Instruction::Pop)
            .end_intermediate(); // is_uncommitted
    }

    fn visit_lookup(&mut self, lookup: &ir::Lookup) {
        let setup = self.make_label("setup");
        let enter = self.make_label("enter_lookup");
        let reenter = self.make_label("reenter_lookup");
        let cleanup = self.make_label("cleanup");

        self.instruction(Instruction::Uncons)
            // The first field of the state is whether we've done set up already or not. If not,
            // then let's do the setup.
            .cond_jump(&setup);

        // If things are already ready to go, it's a matter of calling the query.
        self.label(&enter)
            // Begin by unbinding all the variables of all parameters, just because it's convenient to
            // do up front in one shot.
            .instruction(Instruction::Uncons)
            .label(&reenter);
        let bindset = self.intermediate();
        self.instruction(Instruction::LoadLocal(bindset));
        self.unbind(
            lookup
                .patterns
                .iter()
                .flat_map(|pat| pat.bindings())
                .collect(),
        );
        // Then we do the actual iteration of the lookup.
        self.instruction(Instruction::Copy)
            .raw_call(0)
            .instruction(Instruction::Copy)
            .atom("done")
            .instruction(Instruction::ValNeq)
            .cond_jump(&cleanup)
            .instruction(Instruction::Copy)
            .instruction(Instruction::TypeOf)
            .atom("struct")
            .instruction(Instruction::ValEq)
            .cond_jump(RUNTIME_TYPE_ERROR)
            .instruction(Instruction::Destruct)
            .atom("next")
            .instruction(Instruction::ValEq)
            .cond_jump(RUNTIME_TYPE_ERROR)
            .intermediate(); // return value

        // If the iterator has yielded something, we have to destructure it into all the variables
        // of the unbound parameters.
        let try_again = self.make_label("try_again");
        for pattern in lookup.patterns.iter().rev() {
            self.instruction(Instruction::Uncons);
            // If the pattern doesn't match, the next call of the rule might, so try again still.
            self.context.pattern_match(pattern, &try_again);
        }
        self.end_intermediate() // return value
            .end_intermediate() // bindset
            // Discard the now empty yielded return value
            .instruction(Instruction::Pop)
            // Reconstruct the bindset:state
            .instruction(Instruction::Cons)
            // Put the marker back in too
            .constant(true)
            .instruction(Instruction::Cons)
            // And we're done!
            .bubble(|context| {
                // To try again, just clean up this partial return value and start over.
                context
                    .label(&try_again)
                    .instruction(Instruction::Pop)
                    .jump(&reenter)
                    // In the failure case, just reconstruct the state too
                    .label(cleanup)
                    // Discard the invalid return value
                    .instruction(Instruction::Pop)
                    // Reattach the bindset:state
                    .instruction(Instruction::Cons)
                    // Put the marker back in too
                    .constant(true)
                    .instruction(Instruction::Cons)
                    // And then call it failure
                    .fail();

                // The setup of a lookup is to evaluate all the patterns and call the closure
                // that's currently in the state field, which will return an iterator that is
                // the actual query's state.
                context
                    .label(setup)
                    // First detach the bindset
                    .instruction(Instruction::Uncons)
                    .intermediate(); // bindset
                context.intermediate(); // rule closure

                // Then do the evaluation. Patterns with unbound variables will evaluate to
                // being unbound, as they are being used as output parameters at this time.
                for pattern in &lookup.patterns {
                    context.evaluate_or_skip(pattern).intermediate();
                }
                // Then we do the call, with all those arguments
                context.call_rule(lookup.patterns.len());
                // Clean up the scope
                for _ in &lookup.patterns {
                    context.end_intermediate();
                }
                context
                    .end_intermediate() // rule closure
                    .end_intermediate() // bindset
                    // Bindset gets reattached to scope
                    .instruction(Instruction::Cons)
                    // Then continue as if we didn't just set up
                    .jump(&enter);
            });
    }
}
