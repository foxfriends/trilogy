use crate::{codegen::Codegen, types::CALLABLE_CONTINUATION};
use inkwell::{
    AddressSpace, IntPredicate,
    llvm_sys::LLVMCallConv,
    module::Linkage,
    values::{
        BasicMetadataValueEnum, BasicValue, FunctionValue, InstructionValue, IntValue,
        LLVMTailCallKind, PointerValue,
    },
};

impl<'ctx> Codegen<'ctx> {
    /// Checks whether a value that is supposedly a closure is actually a closure, or if it is
    /// just the value `NO_CLOSURE = 0`.
    fn is_closure(&self, closure: PointerValue<'ctx>) -> IntValue<'ctx> {
        let has_closure = self
            .builder
            .build_ptr_to_int(
                closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None),
                "",
            )
            .unwrap();
        self.builder
            .build_int_compare(
                IntPredicate::NE,
                has_closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
                    .const_zero(),
                "",
            )
            .unwrap()
    }

    fn get_callable_closure(&self, callable: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let bound_closure = self.allocate_value("");
        self.trilogy_callable_closure_into(bound_closure, callable, "");
        bound_closure
    }

    /// Constructs a TrilogyValue that represents the current continuation.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    fn construct_current_continuation(&self) -> (FunctionValue<'ctx>, PointerValue<'ctx>) {
        let continuation_function = self.add_continuation("");
        let continuation = self.allocate_value("cc");
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");

        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.end_continuation_point_as_close(
            closure.as_instruction_value().unwrap(),
            self.builder.get_current_debug_location().unwrap(),
        );
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            yield_to,
            closure,
            continuation_function,
        );

        (continuation_function, continuation)
    }

    /// Calls a standard callable (function or procedure).
    ///
    /// As per standard calling convention, it's basically `call/cc`:
    /// 1. The `return_to` is a newly constructed continuation, representing the current continuation ("returning" is done by `call_continuation`)
    /// 2. The `yield_to` and `end_to` pointers are maintained from the calling context
    /// 3. The arguments are provided
    /// 4. If the callable was a closure, the captures are pulled from the callable object.
    ///
    /// As per the general calling convention, the callable object itself is destroyed, and all
    /// arguments are moved into the call.
    fn call_callable(
        &self,
        value: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
        function_ptr: PointerValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
    ) -> PointerValue<'ctx> {
        let arity = arguments.len();

        // Values must be extracted before they are invalidated
        let bound_closure = self.get_callable_closure(callable);
        let end_to = self.get_end("");
        let yield_to = self.get_yield("");

        // All variables and values are invalid after this point.
        let (continuation_function, current_continuation) = self.construct_current_continuation();

        let mut args = Vec::with_capacity(arity + 4);
        args.extend([current_continuation, yield_to, end_to]);
        args.extend_from_slice(arguments);

        let has_closure = self.is_closure(bound_closure);
        let direct_block = self
            .context
            .append_basic_block(self.get_function(), "call.definition");
        let closure_block = self
            .context
            .append_basic_block(self.get_function(), "call.closure");
        self.builder
            .build_conditional_branch(has_closure, closure_block, direct_block)
            .unwrap();

        self.builder.position_at_end(direct_block);
        let args_loaded: Vec<_> = args
            .iter()
            .map(|arg| {
                self.builder
                    .build_load(self.value_type(), *arg, "")
                    .unwrap()
                    .into()
            })
            .collect();
        let call = self
            .builder
            .build_indirect_call(
                self.procedure_type(arity, false),
                function_ptr,
                &args_loaded,
                "",
            )
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();

        self.builder.position_at_end(closure_block);
        args.push(bound_closure);
        let args_loaded: Vec<_> = args
            .iter()
            .map(|arg| {
                self.builder
                    .build_load(self.value_type(), *arg, "")
                    .unwrap()
                    .into()
            })
            .collect();
        self.trilogy_value_destroy(value);
        let call = self
            .builder
            .build_indirect_call(
                self.procedure_type(arity, true),
                function_ptr,
                &args_loaded,
                "",
            )
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();

        let entry = self
            .context
            .append_basic_block(continuation_function, "entry");
        self.builder.position_at_end(entry);
        self.transfer_debug_info(continuation_function);
        self.get_continuation("")
    }

    /// Calls a procedure value with the provided arguments.
    ///
    /// See `call_callable` for more information on the calling convention.
    pub(crate) fn call_procedure(
        &self,
        procedure: PointerValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(procedure, "");
        let arity = arguments.len();
        let function = self.trilogy_procedure_untag(callable, arity, "");
        self.call_callable(procedure, callable, function, arguments)
    }

    /// Applies a function value to the provided argument. This may also be used to call
    /// a continuation value, as continuations and functions appear identically in Trilogy
    /// source code.
    ///
    /// See `call_callable` for more information on the calling convention.
    pub(crate) fn apply_function(
        &self,
        function: PointerValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(function, "");
        let tag = self.get_callable_tag(callable, "");
        let is_continuation = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.tag_type().const_int(CALLABLE_CONTINUATION, false),
                "",
            )
            .unwrap();

        let call_continuation = self
            .context
            .append_basic_block(self.get_function(), "ap.cont");
        let call_function = self
            .context
            .append_basic_block(self.get_function(), "ap.func");
        self.builder
            .build_conditional_branch(is_continuation, call_continuation, call_function)
            .unwrap();

        self.builder.position_at_end(call_continuation);
        self.call_continuation(function, argument);

        self.builder.position_at_end(call_function);
        let function = self.trilogy_function_untag(callable, "");
        self.call_callable(function, callable, function, &[argument])
    }

    /// Applies a continuation value to the provided argument.
    ///
    /// As per continuation calling convention:
    /// 1. The `return_to` and `yield_to` pointers are stored in the continuation object
    /// 2. The `end_to` pointer is always pulled from the calling context
    /// 3. The argument is provided
    /// 4. The closure is created from the current scope
    ///
    /// The continuation's code itself knows where it is to continue to, so as usual we do not need
    /// to provide that information. The current basic block is terminated, as there is no returning
    /// from a continuation call.
    ///
    /// As per the general calling convention, the continuation object itself is destroyed, and all
    /// arguments are moved into the call.
    pub(crate) fn call_continuation(
        &self,
        function: PointerValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) {
        let callable = self.trilogy_callable_untag(function, "");
        let continuation = self.trilogy_continuation_untag(callable, "");

        let return_to = self.allocate_value("");
        let yield_to = self.allocate_value("");
        self.trilogy_callable_return_to_into(return_to, callable);
        self.trilogy_callable_yield_to_into(yield_to, callable);

        let args: Vec<_> = [
            return_to,
            yield_to,
            self.get_end(""),
            argument,
            self.get_callable_closure(callable),
        ]
        .iter()
        .map(|val| {
            self.builder
                .build_load(self.value_type(), *val, "")
                .unwrap()
                .into()
        })
        .collect();

        self.trilogy_value_destroy(function);

        // NOTE: cleanup will be inserted here
        let call = self
            .builder
            .build_indirect_call(self.continuation_type(), continuation, &args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        let call = call
            .try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap();
        self.end_continuation_point_as_clean(call);
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn continue_to_handled(
        &self,
        function: FunctionValue<'ctx>,
        yield_to: PointerValue<'ctx>,
        cancel_to_closure: PointerValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) {
        let return_to = self.get_return("");
        let end_to = self.get_end("");

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards

        let args: Vec<_> = [return_to, yield_to, end_to, argument, cancel_to_closure]
            .iter()
            .map(|val| {
                self.builder
                    .build_load(self.value_type(), *val, "")
                    .unwrap()
                    .into()
            })
            .collect();

        let call = self.builder.build_direct_call(function, &args, "").unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn continue_to(
        &self,
        function: FunctionValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let end_to = self.get_end("");

        let parent_closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards

        let args: Vec<_> = [return_to, yield_to, end_to, argument, parent_closure]
            .iter()
            .map(|val| {
                self.builder
                    .build_load(self.value_type(), *val, "")
                    .unwrap()
                    .into()
            })
            .collect();

        let call = self.builder.build_direct_call(function, &args, "").unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();
        parent_closure.as_instruction_value().unwrap()
    }

    pub(crate) fn call_main(&self, value: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let chain_function = self.module.add_function(
            "main.return",
            self.continuation_type(),
            Some(Linkage::Private),
        );

        let yield_function = self.module.add_function(
            "main.unhandled_effect",
            self.continuation_type(),
            Some(Linkage::Private),
        );

        let end_function =
            self.module
                .add_function("main.end", self.continuation_type(), Some(Linkage::Private));

        let callable = self.trilogy_callable_untag(value, "");
        let function = self.trilogy_procedure_untag(callable, 0, "");
        let return_continuation = self.allocate_value("return");
        let yield_continuation = self.allocate_value("yield");
        let end_continuation = self.allocate_value("end");

        let return_closure = self.allocate_value("");
        let yield_closure = self.allocate_value("");
        let end_closure = self.allocate_value("");
        self.trilogy_array_init_cap(return_closure, 0, "");
        self.trilogy_array_init_cap(yield_closure, 0, "");
        self.trilogy_array_init_cap(end_closure, 0, "");
        self.trilogy_callable_init_cont(
            return_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            return_closure,
            chain_function,
        );
        self.trilogy_callable_init_cont(
            yield_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            yield_closure,
            yield_function,
        );
        self.trilogy_callable_init_cont(
            end_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            end_closure,
            end_function,
        );

        let args: Vec<_> = [return_continuation, yield_continuation, end_continuation]
            .iter()
            .map(|arg| {
                self.builder
                    .build_load(self.value_type(), *arg, "")
                    .unwrap()
                    .into()
            })
            .collect();
        self.trilogy_value_destroy(value);
        let call = self
            .builder
            .build_indirect_call(self.procedure_type(0, false), function, &args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        self.builder.build_return(None).unwrap();

        let entry = self.context.append_basic_block(yield_function, "entry");
        self.builder.position_at_end(entry);
        let effect = self.get_continuation("");
        _ = self.trilogy_unhandled_effect(effect);

        let entry = self.context.append_basic_block(end_function, "entry");
        self.builder.position_at_end(entry);
        _ = self.trilogy_execution_ended();

        let entry = self.context.append_basic_block(chain_function, "entry");
        self.builder.position_at_end(entry);
        self.get_continuation("")
    }

    pub(crate) fn call_internal(
        &self,
        target: PointerValue<'ctx>,
        procedure: FunctionValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) {
        let mut args = vec![target.into()];
        args.extend_from_slice(arguments);
        let call = self.builder.build_call(procedure, &args, "").unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
    }
}
