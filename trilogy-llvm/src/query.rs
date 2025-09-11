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

        let next_function = self.add_next_to_continuation(0, "iterator_next");
        let brancher = self.branch_continuation_point();
        let next_to =
            self.capture_current_continuation(next_function, &brancher, "next_continuation");
        let next_continuation_cp = self.hold_continuation_point();

        self.compile_query(&query.value, next_to, done_to, &mut bound_ids)?;

        self.become_continuation_point(next_continuation_cp);
        self.begin_next_function(next_function);
        // The `next_to` function of an iterator is called with a function that starts the next
        // iteration of the loop, followed by the values of its arguments, now fully bound. We
        // only the next-iteration function is guaranteed, and we return it here as if it was
        // the "return value" of the iterator. The Lookup case below handles the arguments, if
        // necessary, as that's the only situation where they are possible.
        Some(self.get_continuation("next_iteration"))
    }

    fn compile_query(
        &self,
        query: &ir::QueryValue,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
        bound_ids: &mut Vec<Id>,
    ) -> Option<()> {
        match query {
            ir::QueryValue::Pass => {
                let next_iteration = self.add_continuation("pass_next");
                let brancher = self.branch_continuation_point();
                let next_iteration_continuation =
                    self.capture_current_continuation(next_iteration, &brancher, "pass_next");
                let next_iteration_cp = self.hold_continuation_point();
                self.call_known_continuation(next_to, next_iteration_continuation);

                self.become_continuation_point(next_iteration_cp);
                self.begin_next_function(next_iteration);
                let done_to = self.use_temporary(done_to).unwrap();
                self.void_call_continuation(done_to);
            }
            ir::QueryValue::End => {
                let done_to = self.use_temporary(done_to).unwrap();
                self.void_call_continuation(done_to);
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

                let done_to = self.use_temporary(done_to).unwrap();
                let (next_iteration_inner, output_arguments) =
                    self.call_rule(rule, &arguments, done_to, "lookup_next");
                self.bind_temporary(next_iteration_inner);

                let bound_before_lookup = bound_ids.len();
                for (pattern, out_value) in lookup.patterns.iter().zip(output_arguments) {
                    let out_value = self.use_temporary(out_value).unwrap();
                    bound_ids.extend(self.compile_pattern_match(
                        pattern,
                        out_value,
                        self.get_end(""),
                    )?);
                }

                // Wrap the next iteration with our own, as a lookup requires some cleanup
                // before starting its next internal iteration.
                let next_iteration_with_cleanup = self.add_continuation("rule_query_cleanup");
                let brancher = self.branch_continuation_point();
                let next_iteration_with_cleanup_continuation = self.capture_current_continuation(
                    next_iteration_with_cleanup,
                    &brancher,
                    "pass_next",
                );
                let next_iteration_with_cleanup_cp = self.hold_continuation_point();
                self.call_known_continuation(
                    self.use_temporary(next_to).unwrap(),
                    next_iteration_with_cleanup_continuation,
                );
                // The cleanup: destroy all variables that were unbound on the way in. This uses
                // very similar detection as with patterns, noting that which variables are bound
                // at iteration time can be determined statically as we make the pass through the
                // query's IR.
                self.become_continuation_point(next_iteration_with_cleanup_cp);
                self.begin_next_function(next_iteration_with_cleanup);
                for id in bound_ids[bound_before_lookup..]
                    .iter()
                    .filter(|id| !bound_ids[0..bound_before_lookup].contains(id))
                {
                    let var = self.get_variable(id).unwrap().ptr();
                    self.trilogy_value_destroy(var);
                }
                let next_iteration = self.use_temporary(next_iteration_inner).unwrap();
                self.call_known_continuation(next_iteration, self.get_continuation(""));
            }
            _ => todo!(),
        }
        Some(())
    }
}
