use crate::Codegen;
use inkwell::{AddressSpace, values::PointerValue};
use trilogy_ir::visitor::{HasBindings, HasCanEvaluate};
use trilogy_ir::{Id, ir};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_iterator(
        &self,
        query: &ir::Query,
        done_to: PointerValue<'ctx>,
    ) -> Option<PointerValue<'ctx>> {
        let mut bound_ids = Vec::default();

        for variable in query.value.bindings() {
            self.variable(&variable);
        }

        self.compile_query(query, done_to, &mut bound_ids)
    }

    fn compile_query(
        &self,
        query: &ir::Query,
        done_to: PointerValue<'ctx>,
        bound_ids: &mut Vec<Id>,
    ) -> Option<PointerValue<'ctx>> {
        let next_function = self.add_next_to_continuation(0, "iterator_next");
        let (next_to, then_continuation_cp) =
            self.capture_current_continuation(next_function, "next_continuation");

        self.compile_query_value(&query.value, next_to, done_to, bound_ids)?;

        self.become_continuation_point(then_continuation_cp);
        self.begin_next_function(next_function);
        // The `next_to` function of an iterator is called with a function that starts the next
        // iteration of the loop, followed by the values of its arguments, now fully bound. We
        // only the next-iteration function is guaranteed, and we return it here as if it was
        // the "return value" of the iterator. The Lookup case below handles the arguments, if
        // necessary, as that's the only situation where they are possible.
        Some(self.get_continuation("next_iteration"))
    }

    fn compile_query_value(
        &self,
        query: &ir::QueryValue,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
        bound_ids: &mut Vec<Id>,
    ) -> Option<()> {
        match query {
            ir::QueryValue::Pass => {
                self.next_deterministic(next_to, done_to, "pass_next");
            }
            ir::QueryValue::End => {
                let done_to = self.use_temporary(done_to).unwrap();
                self.void_call_continuation(done_to);
            }
            ir::QueryValue::Is(expr) => {
                let condition = self.compile_expression(expr, "is.condition")?;

                let original_function_scope = self.get_function();
                let if_true_block = self
                    .context
                    .append_basic_block(original_function_scope, "is.true");
                let if_false_block = self
                    .context
                    .append_basic_block(original_function_scope, "is.false");
                let cond_bool = self.trilogy_boolean_untag(condition, "is.bool");
                self.trilogy_value_destroy(condition);
                self.builder
                    .build_conditional_branch(cond_bool, if_true_block, if_false_block)
                    .unwrap();
                let true_cp = self.branch_continuation_point();

                self.builder.position_at_end(if_false_block);
                self.void_call_continuation(self.use_temporary(done_to).unwrap());

                self.become_continuation_point(true_cp);
                self.builder.position_at_end(if_true_block);
                self.next_deterministic(next_to, done_to, "is_next");
            }
            ir::QueryValue::Lookup(lookup) if lookup.patterns.is_empty() => {
                let rule = self.compile_expression(&lookup.path, "rule")?;
                let (next_iteration, out) = self.call_rule(rule, &[], done_to, "lookup_next");
                assert!(out.is_empty());
                let next_to = self.use_temporary(next_to).unwrap();
                self.call_known_continuation(next_to, next_iteration);
            }
            ir::QueryValue::Lookup(lookup) => {
                // NOTE: at this time, the lookup expression is not an arbitrary expression, only a
                // reference/module path, so branching is not possible here.
                let rule = self.compile_expression(&lookup.path, "rule")?;
                // NOTE: similarly, the arguments of a query must be syntactically both patterns and
                // expressions, so we can again guarantee that branching is not possible.
                let arguments = lookup
                    .patterns
                    .iter()
                    .map(|pattern| {
                        if !pattern.can_evaluate() {
                            // This pattern is not possibly an expression, e.g. due to containing a wildcard
                            // or other pattern-only syntax element. It can only be used as an output parameter.
                            let arg = self.allocate_undefined("out_arg");
                            self.bind_temporary(arg);
                            return Some(arg);
                        }
                        let variables = pattern
                            .bindings()
                            .iter()
                            .map(|var| self.get_variable(var).unwrap())
                            .collect::<Vec<_>>();
                        if variables.is_empty() {
                            let arg = self.compile_expression(pattern, "in_arg")?;
                            self.bind_temporary(arg);
                            return Some(arg);
                        }
                        // This pattern is (potentially) fully defined. We must confirm by checking all
                        // the variables at runtime, and if they are all set, then we can construct
                        // this pattern.
                        let out_block = self.context.append_basic_block(self.get_function(), "out");
                        let original_function = self.get_function();
                        let snapshot = self.snapshot_function_context();
                        for variable in variables {
                            let next_arg =
                                self.context.append_basic_block(self.get_function(), "next");
                            self.branch_undefined(variable.ptr(), out_block, next_arg);
                            self.builder.position_at_end(next_arg);
                        }
                        let in_arg = self.compile_expression(pattern, "in_arg")?;
                        let final_function = self.get_function();
                        if original_function == final_function {
                            let merge_block =
                                self.context.append_basic_block(final_function, "merge_arg");

                            let in_block = self.builder.get_insert_block().unwrap();
                            self.builder
                                .build_unconditional_branch(merge_block)
                                .unwrap();

                            self.builder.position_at_end(out_block);
                            let out_arg = self.allocate_undefined("out_arg");
                            self.builder
                                .build_unconditional_branch(merge_block)
                                .unwrap();

                            self.builder.position_at_end(merge_block);
                            let phi = self
                                .builder
                                .build_phi(self.context.ptr_type(AddressSpace::default()), "arg")
                                .unwrap();
                            phi.add_incoming(&[(&in_arg, in_block), (&out_arg, out_block)]);
                            let arg = phi.as_basic_value().into_pointer_value();
                            self.bind_temporary(arg);
                            Some(arg)
                        } else {
                            let arg_continuation = self.add_continuation("arg");
                            let end = self.continue_in_scope(arg_continuation, in_arg);
                            self.end_continuation_point_as_close(end);

                            self.restore_function_context(snapshot);
                            self.builder.position_at_end(out_block);
                            let end = self.void_continue_in_scope(arg_continuation);
                            self.end_continuation_point_as_close(end);

                            self.begin_next_function(arg_continuation);
                            let arg = self.get_continuation("arg");
                            self.bind_temporary(arg);
                            Some(arg)
                        }
                    })
                    .collect::<Option<Vec<_>>>()?;

                let (next_iteration_inner, output_arguments) = self.call_rule(
                    rule,
                    &arguments,
                    self.use_temporary(done_to).unwrap(),
                    "lookup_next",
                );
                self.bind_temporary(next_iteration_inner);

                // Wrap the next iteration with our own, as a lookup requires some cleanup
                // before starting its next internal iteration.
                let next_iteration_with_cleanup = self.add_continuation("lookup.next");
                let (next_iteration_with_cleanup_continuation, next_iteration_with_cleanup_cp) =
                    self.capture_current_continuation(next_iteration_with_cleanup, "lookup.next");

                let bound_before_lookup = bound_ids.len();
                for (pattern, out_value) in lookup.patterns.iter().zip(output_arguments) {
                    let out_value = self.use_temporary(out_value).unwrap();
                    self.compile_pattern_match_with_bindings(
                        pattern,
                        out_value,
                        next_iteration_with_cleanup_continuation,
                        bound_ids,
                    )?;
                }

                self.call_known_continuation(
                    self.use_temporary(next_to).unwrap(),
                    next_iteration_with_cleanup_continuation,
                );

                self.become_continuation_point(next_iteration_with_cleanup_cp);
                self.begin_next_function(next_iteration_with_cleanup);
                self.cleanup_go_next(next_iteration_inner, bound_ids, bound_before_lookup);
            }
            ir::QueryValue::Disjunction(disj) => {
                let disj_second_fn = self.add_continuation("disj.second");
                let (disj_second, disj_second_cp) =
                    self.capture_current_continuation(disj_second_fn, "disj.second");
                let next_of_first =
                    self.compile_query(&disj.0, disj_second, &mut bound_ids.clone())?;
                self.call_known_continuation(self.use_temporary(next_to).unwrap(), next_of_first);

                self.become_continuation_point(disj_second_cp);
                self.begin_next_function(disj_second_fn);
                let next_of_second = self.compile_query(&disj.1, done_to, bound_ids)?;
                self.call_known_continuation(self.use_temporary(next_to).unwrap(), next_of_second);
            }
            ir::QueryValue::Conjunction(conj) => {
                let next_of_first = self.compile_query(&conj.0, done_to, bound_ids)?;
                self.bind_temporary(next_of_first);
                let next_of_second = self.compile_query(&conj.1, next_of_first, bound_ids)?;
                let next_to = self.use_temporary(next_to).unwrap();
                self.call_known_continuation(next_to, next_of_second);
            }
            ir::QueryValue::Implication(implication) => {
                self.compile_query(&implication.0, done_to, bound_ids)?;
                let next_of_second = self.compile_query(&implication.1, done_to, bound_ids)?;
                let next_to = self.use_temporary(next_to).unwrap();
                self.call_known_continuation(next_to, next_of_second);
            }
            ir::QueryValue::Alternative(..) => todo!(),
            ir::QueryValue::Direct(unification) => {
                let rvalue = self.compile_expression(&unification.expression, "rvalue")?;
                let pre_len = bound_ids.len();
                self.compile_pattern_match_with_bindings(
                    &unification.pattern,
                    rvalue,
                    done_to,
                    bound_ids,
                )?;
                self.next_cleanup(done_to, next_to, bound_ids, pre_len, "assign_next");
            }
            ir::QueryValue::Element(..) => todo!(),
            ir::QueryValue::Not(..) => todo!(),
        }
        Some(())
    }

    fn next_deterministic(
        &self,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
        name: &str,
    ) {
        let next_iteration = self.add_continuation(name);
        let (next_iteration_continuation, next_iteration_cp) =
            self.capture_current_continuation(next_iteration, name);
        let next_to = self.use_temporary(next_to).unwrap();
        self.call_known_continuation(next_to, next_iteration_continuation);

        self.become_continuation_point(next_iteration_cp);
        self.begin_next_function(next_iteration);
        let done_to = self.use_temporary(done_to).unwrap();
        self.void_call_continuation(done_to);
    }

    fn next_cleanup(
        &self,
        next_iteration: PointerValue<'ctx>,
        next_to: PointerValue<'ctx>,
        bound_ids: &[Id],
        keep_ids: usize,
        name: &str,
    ) {
        // Wrap the next iteration with our own, as a lookup requires some cleanup
        // before starting its next internal iteration.
        let next_iteration_with_cleanup = self.add_continuation(name);
        let (next_iteration_with_cleanup_continuation, next_iteration_with_cleanup_cp) =
            self.capture_current_continuation(next_iteration_with_cleanup, name);
        self.call_known_continuation(
            self.use_temporary(next_to).unwrap(),
            next_iteration_with_cleanup_continuation,
        );

        self.become_continuation_point(next_iteration_with_cleanup_cp);
        self.begin_next_function(next_iteration_with_cleanup);
        self.cleanup_go_next(next_iteration, bound_ids, keep_ids);
    }

    fn cleanup_go_next(
        &self,
        next_iteration: PointerValue<'ctx>,
        bound_ids: &[Id],
        keep_ids: usize,
    ) {
        // The cleanup: destroy all variables that were unbound on the way in. This uses
        // very similar detection as with patterns, noting that which variables are bound
        // at iteration time can be determined statically as we make the pass through the
        // query's IR.
        for id in bound_ids[keep_ids..]
            .iter()
            .filter(|id| !bound_ids[0..keep_ids].contains(id))
        {
            let var = self.get_variable(id).unwrap().ptr();
            self.trilogy_value_destroy(var);
        }
        let next_iteration = self.use_temporary(next_iteration).unwrap();
        self.void_call_continuation(next_iteration);
    }
}
