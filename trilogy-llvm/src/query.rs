use crate::{Codegen, IMPLICIT_PARAMS};
use inkwell::{AddressSpace, values::PointerValue};
use trilogy_ir::{
    ir,
    visitor::{HasBindings, HasCanEvaluate},
};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_iterator(
        &self,
        query: &ir::Query,
        done_to: PointerValue<'ctx>,
    ) -> Option<PointerValue<'ctx>> {
        for variable in query.value.bindings() {
            self.variable(&variable);
        }

        let next_function = self.add_continuation("next");
        let brancher = self.end_continuation_point_as_branch();
        let next_to =
            self.capture_current_continuation(next_function, &brancher, "next_continuation");
        let next_continuation_cp = self.hold_continuation_point();

        self.compile_query(&query.value, next_to, done_to)?;

        self.become_continuation_point(next_continuation_cp);
        self.begin_next_function(next_function);
        Some(self.get_continuation("next_iteration"))
    }

    fn compile_query(
        &self,
        query: &ir::QueryValue,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
    ) -> Option<()> {
        match query {
            ir::QueryValue::Pass => {
                let next_iteration = self.add_continuation("pass_next");
                let brancher = self.end_continuation_point_as_branch();
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
                let next_iteration = self.call_rule(rule, &[], done_to, "lookup_next");
                let next_to = self.use_temporary(next_to).unwrap();
                self.call_known_continuation(next_to, next_iteration);
            }
            ir::QueryValue::Lookup(lookup) => {
                let rule = self.compile_expression(&lookup.path, "rule")?;
                let mut arguments = lookup
                    .patterns
                    .iter()
                    .map(|pattern| {
                        if !pattern.can_evaluate() {
                            // This pattern is not possibly an expression, e.g. due to containing a wildcard
                            // or other pattern-only syntax element. It can only be used as an output parameter.
                            return Some(self.allocate_undefined("out_arg"));
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
                            let in_block = self.builder.get_insert_block().unwrap();
                            self.builder.position_at_end(out_block);
                            let out_arg = self.allocate_undefined("out_arg");
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
                for argument in &mut arguments {
                    *argument = self.use_temporary(*argument).unwrap();
                }

                let next_iteration_inner = self.call_rule(rule, &arguments, done_to, "lookup_next");
                for (n, param) in lookup.patterns.iter().enumerate() {
                    let value = self.function_params.borrow()[n + 1 /* extra one is the "next iteration" */ + IMPLICIT_PARAMS];
                    self.compile_pattern_match(param, value, self.get_end(""))?
                }
                let next_to = self.use_temporary(next_to).unwrap();
                self.call_known_continuation(next_to, next_iteration_inner);
            }
            _ => todo!(),
        }
        Some(())
    }
}
