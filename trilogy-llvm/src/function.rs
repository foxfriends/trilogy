use crate::Codegen;
use inkwell::values::{BasicValue, FunctionValue, GlobalValue};
use source_span::Span;
use std::borrow::Borrow;
use trilogy_ir::Id;
use trilogy_ir::ir::{self, Value};

impl<'ctx> Codegen<'ctx> {
    fn write_function_accessor(
        &self,
        definition: &ir::FunctionDefinition,
        accessor: FunctionValue<'ctx>,
        accessing: FunctionValue<'ctx>,
    ) -> GlobalValue<'ctx> {
        let has_context = accessor.count_params() == 2;
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        let metadata = self.build_callable_data(
            &self.module_path(),
            &definition.name.to_string(),
            definition.overloads[0].parameters.len() as u32,
            definition.span(),
            None,
        );
        if has_context {
            let ctx = accessor.get_nth_param(1).unwrap().into_pointer_value();
            self.trilogy_callable_init_do(sret, 1, ctx, accessing, metadata);
        } else {
            self.trilogy_callable_init_proc(sret, 1, accessing, metadata);
        }
        self.builder.build_return(None).unwrap();
        metadata
    }

    pub(crate) fn compile_function(
        &self,
        definition: &ir::FunctionDefinition,
        module_context: Option<Vec<Id>>,
    ) {
        let name = definition.name.to_string();
        let accessor_name = format!("{}::{}", self.module_path(), name);
        let accessor = self.module.get_function(&accessor_name).unwrap();
        let function = self.add_function(&name, &name, definition.span(), false);
        let metadata = self.write_function_accessor(definition, accessor, function);
        self.set_current_definition(
            name.clone(),
            name,
            definition.span(),
            metadata,
            module_context,
        );
        self.compile_function_body(function, &definition.overloads, definition.span());
        self.close_continuation();
    }

    pub(crate) fn compile_function_body(
        &self,
        function: FunctionValue<'ctx>,
        overloads: &[impl Borrow<ir::Function>],
        span: Span,
    ) {
        self.begin_function(function, span);

        let arity = overloads[0].borrow().parameters.len();
        let mut params = Vec::with_capacity(arity);
        for i in 0..arity as u32 - 1 {
            let continuation = self.add_continuation(&format!("param_{}", i + 1));
            params.push(self.get_continuation_temporary());
            let return_to = self.get_return("return");
            let cont_val = self.allocate_value("next_call");

            let here = self.get_current_definition();
            let child_metadata = self.build_callable_data(
                &self.module_path(),
                &here.name,
                arity as u32 - i,
                span,
                Some(here.metadata),
            );

            let closure = self
                .builder
                .build_alloca(self.value_type(), "TEMP_CLOSURE")
                .unwrap();
            self.trilogy_callable_init_do(cont_val, 1, closure, continuation, child_metadata);
            let inner_cp = self.capture_contination_point(closure.as_instruction_value().unwrap());
            self.call_known_continuation(return_to, cont_val);

            self.become_continuation_point(inner_cp);
            self.begin_next_function(continuation);
        }

        // The last parameter is collected in the same continuation as the body
        params.push(self.get_continuation_temporary());

        'outer: for (i, overload) in overloads.iter().enumerate() {
            let overload = overload.borrow();
            assert_eq!(overload.parameters.len(), arity);
            if matches!(
                overload.guard.as_ref().map(|g| &g.value),
                Some(Value::Boolean(false))
            ) {
                continue;
            }

            self.set_span(overload.head_span);

            let next_overload_function = self.add_continuation(if i == overloads.len() - 1 {
                "failed"
            } else {
                ""
            });
            let (go_to_next_overload, next_overload_cp) = self.capture_current_continuation(
                next_overload_function,
                "next_overload",
                overload.span
            );

            for (pattern, param) in overload.parameters.iter().zip(&params) {
                if self
                    .compile_pattern_match(pattern, *param, go_to_next_overload)
                    .is_none()
                {
                    break 'outer;
                }
            }

            if let Some(guard) = &overload.guard
                && !matches!(guard.value, Value::Boolean(true))
            {
                let Some(guard_bool) = self.compile_expression(guard, "match.guard") else {
                    self.become_continuation_point(next_overload_cp);
                    self.begin_next_function(next_overload_function);
                    continue;
                };
                let guard_flag = self.trilogy_boolean_untag(guard_bool, "");
                self.trilogy_value_destroy(guard_bool); // NOTE: bool doesn't really need to be destroyed... but do it anyway
                let next_block = self.context.append_basic_block(self.get_function(), "next");
                let body_block = self.context.append_basic_block(self.get_function(), "body");
                let body_cp = self.branch_continuation_point();
                self.builder
                    .build_conditional_branch(guard_flag, body_block, next_block)
                    .unwrap();

                self.builder.position_at_end(next_block);
                let go_next = self.use_temporary_clone(go_to_next_overload).unwrap();
                self.void_call_continuation(go_next);

                self.builder.position_at_end(body_block);
                self.become_continuation_point(body_cp);
            }

            self.destroy_owned_temporary(go_to_next_overload);

            if let Some(value) = self.compile_expression(&overload.body, "") {
                let ret = self.get_return("");
                self.call_known_continuation(ret, value);
            }

            self.become_continuation_point(next_overload_cp);
            self.begin_next_function(next_overload_function);
        }

        let end = self.get_end("");
        self.void_call_continuation(end);

        self.end_function();
    }
}
