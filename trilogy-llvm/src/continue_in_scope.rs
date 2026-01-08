use crate::codegen::Codegen;
use inkwell::AddressSpace;
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, FunctionValue, InstructionValue, PointerValue,
};
use source_span::Span;

impl<'ctx> Codegen<'ctx> {
    /// Continues to a point in the same lexical scope directly, without any runtime
    /// continuation object. This is typically used within an expression that spans
    /// multiple continuations, such as an `if-else` expression.
    ///
    /// The current basic block is terminated. The current continuation point must be
    /// closed; the instruction from which to terminate it is returned, and should
    /// be passed to some `end_continuation_` function that will close closure pointer
    /// returned.
    #[must_use = "continuation point must be closed"]
    pub(crate) fn continue_in_scope(
        &self,
        function: FunctionValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        self.continue_in_scope_inner(function, self.load_value(argument, "").into())
    }

    /// See `continue_in_scope`; this does that, but passes an `undefined` value as the
    /// parameter, assuming that the continuation we are entering does not refer to the
    /// value at all.
    #[must_use = "continuation point must be closed"]
    pub(crate) fn void_continue_in_scope(
        &self,
        function: FunctionValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        self.continue_in_scope_inner(function, self.value_type().const_zero().into())
    }

    fn continue_in_scope_inner(
        &self,
        function: FunctionValue<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) -> InstructionValue<'ctx> {
        let return_to = self.get_return("");
        let end_to = self.get_end("");
        let yield_to = self.get_yield("");

        let parent_closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            argument,
            self.load_value(parent_closure, "").into(),
        ];

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.direct_tail_call(function, args, "");
        self.builder.build_return(None).unwrap();
        parent_closure.as_instruction_value().unwrap()
    }

    pub(crate) fn continue_in_handler(
        &self,
        function: FunctionValue<'ctx>,
        yield_to: PointerValue<'ctx>,
    ) {
        let return_to = self.get_return("");
        let end_to = self.get_end("");

        let parent_closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.value_type().const_zero().into(),
            self.load_value(parent_closure, "").into(),
        ];

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.direct_tail_call(function, args, "");
        self.builder.build_return(None).unwrap();
        self.end_continuation_point_as_close(parent_closure.as_instruction_value().unwrap());
    }

    pub(crate) fn continue_in_loop(
        &self,
        continue_function: FunctionValue<'ctx>,
        span: Span,
    ) -> PointerValue<'ctx> {
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let end_to = self.get_end("");

        let continue_to = self.allocate_value("continue");
        self.bind_temporary(continue_to);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        let current = self.get_current_definition();
        let metadata = self.build_callable_data(
            &self.module_path(),
            &current.name,
            1,
            span,
            Some(current.metadata),
        );

        let continue_to_callable = self.trilogy_callable_init_cont(
            continue_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            closure,
            continue_function,
            metadata,
        );

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.end_continuation_point_as_close_after(
            closure.as_instruction_value().unwrap(),
            continue_to_callable.as_instruction().unwrap(),
        );

        let closure = self.allocate_value("");
        self.trilogy_callable_closure_into(closure, continue_to_callable, "");

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.value_type().const_zero().into(),
            self.load_value(closure, "").into(),
        ];
        self.direct_tail_call(continue_function, args, "");
        self.builder.build_return(None).unwrap();
        continue_to
    }
}
