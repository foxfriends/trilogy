use std::borrow::Borrow;

use crate::Codegen;
use inkwell::values::{FunctionValue, PointerValue};
use source_span::Span;
use trilogy_ir::{Id, ir};

// NOTE: params start at 7, due to return, yield, end, cancel, resume, break, and continue
const RULE_IMPLICIT_PARAMS: usize = 7;

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

        'outer: for overload in overloads {
            let overload = overload.borrow();
            assert_eq!(overload.parameters.len(), arity);
            self.set_span(overload.head_span);
            for (n, param) in overload.parameters.iter().enumerate() {
                let value = self.function_params.borrow()[n + RULE_IMPLICIT_PARAMS];

                let skip_parameter = self.context.append_basic_block(function, "skip_parameter");
                let bind_parameter = self.context.append_basic_block(function, "bind_parameter");
                self.branch_undefined(value, skip_parameter, bind_parameter);

                self.builder.position_at_end(skip_parameter);
                // TODO: go to next parameter

                self.builder.position_at_end(bind_parameter);
                if self
                    .compile_pattern_match(param, value, self.get_done(""))
                    .is_none()
                {
                    break 'outer;
                }
                // TODO: also go to next parameter
            }
            let next = self.get_next("");
            let done = self.get_done("");
            self.compile_query(&overload.body, next, done);
            // TODO: ^ done should be next overload, except for on the last overload
        }
        self.end_function();
    }

    fn compile_query(
        &self,
        _body: &ir::Query,
        _next: PointerValue<'ctx>,
        _done: PointerValue<'ctx>,
    ) {
        todo!();
    }
}
