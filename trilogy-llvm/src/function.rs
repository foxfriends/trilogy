use crate::Codegen;
use inkwell::values::{BasicValue, FunctionValue};
use source_span::Span;
use std::borrow::Borrow;
use trilogy_ir::{Id, ir};

impl<'ctx> Codegen<'ctx> {
    fn write_function_accessor(
        &self,
        accessor: FunctionValue<'ctx>,
        accessing: FunctionValue<'ctx>,
    ) {
        let has_context = accessor.count_params() == 2;
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        if has_context {
            let ctx = accessor.get_nth_param(1).unwrap().into_pointer_value();
            self.trilogy_callable_init_fn(sret, ctx, accessing);
        } else {
            self.trilogy_callable_init_func(sret, accessing);
        }
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn compile_function(
        &self,
        definition: &ir::FunctionDefinition,
        module_context: Option<Vec<Id>>,
    ) {
        let name = definition.name.to_string();
        let accessor_name = format!("{}::{}", self.module_path(), name);
        let accessor = self.module.get_function(&accessor_name).unwrap();
        let function = self.add_function(
            &name,
            &name,
            definition.span(),
            module_context.is_some(),
            false,
        );
        self.write_function_accessor(accessor, function);
        self.set_current_definition(name.clone(), name, definition.span(), module_context);
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
        for _ in 0..arity as u32 - 1 {
            let continuation = self.add_continuation("");
            let param = self.get_continuation("");
            self.bind_temporary(param);
            params.push(param);
            let return_to = self.get_return("");
            let cont_val = self.allocate_value("");

            let closure = self
                .builder
                .build_alloca(self.value_type(), "TEMP_CLOSURE")
                .unwrap();
            self.trilogy_callable_init_fn(cont_val, closure, continuation);
            let inner_cp = self.capture_contination_point(closure.as_instruction_value().unwrap());
            self.call_known_continuation(return_to, cont_val);

            self.become_continuation_point(inner_cp);
            self.begin_next_function(continuation);
        }

        // The last parameter is collected in the same continuation as the body
        let param = self.get_continuation("");
        self.bind_temporary(param);
        params.push(param);

        'outer: for overload in overloads {
            let overload = overload.borrow();
            assert_eq!(overload.parameters.len(), arity);
            self.set_span(overload.head_span);

            let next_overload_function = self.add_continuation("");
            let (go_to_next_overload, next_overload_cp) =
                self.capture_current_continuation(next_overload_function, "next_overload");

            for (pattern, param) in overload.parameters.iter().zip(&params) {
                if self
                    .compile_pattern_match(pattern, *param, go_to_next_overload)
                    .is_none()
                {
                    break 'outer;
                }
            }

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
