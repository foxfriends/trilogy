use crate::{Codegen, IMPLICIT_PARAMS};
use inkwell::values::FunctionValue;
use source_span::Span;
use std::borrow::Borrow;
use trilogy_ir::{Id, ir};

impl<'ctx> Codegen<'ctx> {
    fn write_rule_accessor(
        &self,
        accessor: FunctionValue<'ctx>,
        accessing: FunctionValue<'ctx>,
        arity: usize,
    ) {
        let has_context = accessor.count_params() == 2;
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        if has_context {
            let ctx = accessor.get_nth_param(1).unwrap().into_pointer_value();
            self.trilogy_callable_init_qy(sret, arity, ctx, accessing);
        } else {
            self.trilogy_callable_init_rule(sret, arity, accessing);
        }
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn compile_rule(
        &self,
        definition: &ir::RuleDefinition,
        module_context: Option<Vec<Id>>,
    ) {
        let arity = definition.overloads[0].parameters.len();
        let name = definition.name.to_string();
        let accessor_name = format!("{}::{}", self.module_path(), name);
        let accessor = self.module.get_function(&accessor_name).unwrap();
        let function = self.add_rule(
            &name,
            arity,
            &name,
            definition.span(),
            module_context.is_some(),
            false,
        );
        self.write_rule_accessor(accessor, function, arity);
        self.set_current_definition(
            name.to_owned(),
            name.to_owned(),
            definition.span(),
            module_context,
        );
        self.compile_rule_body(function, &definition.overloads, definition.span());
        self.close_continuation();
    }

    pub(crate) fn compile_rule_body(
        &self,
        function: FunctionValue<'ctx>,
        overloads: &[impl Borrow<ir::Rule>],
        span: Span,
    ) {
        self.begin_function(function, span);
        let arity = overloads[0].borrow().parameters.len();

        let brancher = self.end_continuation_point_as_branch();

        'outer: for overload in overloads {
            let overload = overload.borrow();
            assert_eq!(overload.parameters.len(), arity);
            self.set_span(overload.head_span);

            let next_overload_function = self.add_continuation("");
            let go_to_next_overload = self.capture_current_continuation(
                next_overload_function,
                &brancher,
                "next_overload",
            );
            let next_overload_cp = self.hold_continuation_point();

            for (n, param) in overload.parameters.iter().enumerate() {
                let value = self.function_params.borrow()[n + IMPLICIT_PARAMS];

                let skip_parameter = self.context.append_basic_block(function, "skip_parameter");
                let bind_parameter = self.context.append_basic_block(function, "bind_parameter");
                self.branch_undefined(value, skip_parameter, bind_parameter);

                let original_insert_function = self.get_function();
                let original_snapshot = self.snapshot_function_context();

                self.builder.position_at_end(bind_parameter);
                if self
                    .compile_pattern_match(
                        param,
                        value,
                        self.use_temporary(go_to_next_overload).unwrap(),
                    )
                    .is_none()
                {
                    break 'outer;
                }
                let post_insert_function = self.get_function();
                if original_insert_function == post_insert_function {
                    // We could do the second case always, but it's much more efficient to skip
                    // making more closures if it can be avoided, which is the case if the pattern
                    // never causes a branch (e.g. often because it's just a single variable).
                    self.builder
                        .build_unconditional_branch(skip_parameter)
                        .unwrap();
                    self.builder.position_at_end(skip_parameter);
                } else {
                    // Otherwise, if the parameter binding caused the continuation to change already,
                    // we need yet another continuation that merges the "skip" case with the
                    // "successful binding" case.
                    let next_parameter_function = self.add_continuation("");
                    let next_closure = self.void_continue_in_scope(next_parameter_function);
                    self.add_branch_end_as_close(&brancher, next_closure);

                    self.restore_function_context(original_snapshot);
                    self.builder.position_at_end(skip_parameter);
                    let skip_closure = self.void_continue_in_scope(next_parameter_function);
                    self.add_branch_end_as_close(&brancher, skip_closure);

                    self.begin_next_function(next_parameter_function);
                }
            }
            let Some(next_iteration) = self.compile_iterator(&overload.body, go_to_next_overload)
            else {
                break 'outer;
            };
            self.bind_temporary(next_iteration);

            let mut arguments = Vec::with_capacity(arity + 1);
            arguments.push(next_iteration);
            for param in &overload.parameters {
                let Some(param_value) = self.compile_expression(param, "") else {
                    break 'outer;
                };
                self.bind_temporary(param_value);
                arguments.push(param_value);
            }
            let next = self.get_next("");
            for arg in arguments.iter_mut() {
                *arg = self.use_temporary(*arg).unwrap();
            }
            self.call_next(next, &arguments);

            self.become_continuation_point(next_overload_cp);
            self.begin_next_function(next_overload_function);
        }
        let done = self.get_done("");
        self.void_call_continuation(done);
        self.end_function();
    }
}
