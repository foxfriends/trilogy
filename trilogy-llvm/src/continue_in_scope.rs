use crate::codegen::Codegen;
use inkwell::AddressSpace;
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, FunctionValue, InstructionValue, PointerValue,
};

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
        let next_to = self.get_next("");
        let done_to = self.get_done("");

        let parent_closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.load_value(next_to, "").into(),
            self.load_value(done_to, "").into(),
            argument,
            self.load_value(parent_closure, "").into(),
        ];

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.direct_tail_call(function, args, "");
        self.builder.build_return(None).unwrap();
        parent_closure.as_instruction_value().unwrap()
    }

    /// When continuing into a handler, we have to promote `return`, `break` and `cancel` with their
    /// current contextual values. This allows calls to those continuations to invisibly discard the
    /// effect handler that we are about to install when called.
    #[must_use = "continuation point must be closed"]
    pub(crate) fn continue_in_handler(
        &self,
        function: FunctionValue<'ctx>,
        yield_to: PointerValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        let return_to = self.get_return("");
        let end_to = self.get_end("");
        let next_to = self.get_next("");
        let done_to = self.get_done("");

        self.trilogy_callable_promote(
            return_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.get_yield(""),
            self.get_next(""),
            self.get_done(""),
        );

        let parent_closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        let args = &[
            self.load_value(return_to, "").into(),
            self.load_value(yield_to, "").into(),
            self.load_value(end_to, "").into(),
            self.load_value(next_to, "").into(),
            self.load_value(done_to, "").into(),
            self.value_type().const_zero().into(),
            self.load_value(parent_closure, "").into(),
        ];

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.direct_tail_call(function, args, "");
        self.builder.build_return(None).unwrap();
        parent_closure.as_instruction_value().unwrap()
    }

    /// When continuing into a loop, promote `return` and `cancel` with their contextual
    /// values so that when called, they jump out of anything we've added inside the loop.
    pub(crate) fn continue_in_loop(
        &self,
        continue_function: FunctionValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let end_to = self.get_end("");
        let next_to = self.get_next("");
        let done_to = self.get_done("");

        self.trilogy_callable_promote(
            return_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.get_yield(""),
            self.get_next(""),
            self.get_done(""),
        );

        let continue_to = self.allocate_value("continue");
        self.bind_temporary(continue_to);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        let continue_to_callable = self.trilogy_callable_init_cont(
            continue_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            closure,
            continue_function,
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
            self.load_value(next_to, "").into(),
            self.load_value(done_to, "").into(),
            self.value_type().const_zero().into(),
            self.load_value(closure, "").into(),
        ];
        self.direct_tail_call(continue_function, args, "");
        self.builder.build_return(None).unwrap();
        continue_to
    }
}
