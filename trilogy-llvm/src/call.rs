use crate::codegen::Codegen;
use inkwell::{
    debug_info::AsDIScope,
    llvm_sys::{debuginfo::LLVMDIFlagPublic, LLVMCallConv},
    module::Linkage,
    values::{
        BasicMetadataValueEnum, BasicValue, FunctionValue, InstructionValue, IntValue,
        LLVMTailCallKind, PointerValue,
    },
    AddressSpace, IntPredicate,
};

impl<'ctx> Codegen<'ctx> {
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

    fn add_continuation(&self) -> FunctionValue<'ctx> {
        let (name, span) = self.get_current_definition();
        let chain_function =
            self.module
                .add_function(&name, self.continuation_type(), Some(Linkage::Private));
        let procedure_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            &name,
            Some(&chain_function.get_name().to_str().unwrap()),
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.continuation_di_type(),
            true,
            false,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        chain_function.set_subprogram(procedure_scope);
        chain_function
    }

    fn get_callable_closure(&self, callable: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let bound_closure = self.allocate_value("");
        self.trilogy_callable_closure_into(bound_closure, callable, "");
        bound_closure
    }

    fn call_callable(
        &self,
        callable: PointerValue<'ctx>,
        function: PointerValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) -> PointerValue<'ctx> {
        let bound_closure = self.get_callable_closure(callable);

        let arity = arguments.len();
        let chain_function = self.add_continuation();

        let continuation = self.allocate_value("cont");

        let mut args = vec![
            self.get_return().into(),
            self.get_yield().into(),
            self.get_end().into(),
            continuation.into(),
        ];
        args.extend_from_slice(arguments);

        let parent_closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.set_continued(parent_closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            self.get_return(),
            self.get_yield(),
            parent_closure,
            chain_function.as_global_value().as_pointer_value(),
        );

        let direct_block = self
            .context
            .append_basic_block(self.get_function(), "call.proc");
        let closure_block = self
            .context
            .append_basic_block(self.get_function(), "call.do");

        let has_closure = self.is_closure(bound_closure);
        self.builder
            .build_conditional_branch(has_closure, closure_block, direct_block)
            .unwrap();

        self.builder.position_at_end(direct_block);
        let call = self
            .builder
            .build_indirect_call(self.procedure_type(arity, false), function, &args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();

        self.builder.position_at_end(closure_block);
        args.push(bound_closure.into());
        let call = self
            .builder
            .build_indirect_call(self.procedure_type(arity, true), function, &args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();

        let entry = self.context.append_basic_block(chain_function, "entry");
        self.builder.position_at_end(entry);
        // TODO: clone debug scopes with new subprogram node
        self.get_continuation()
    }

    pub(crate) fn call_procedure(
        &self,
        value: PointerValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(value, "");
        let arity = arguments.len();
        let function = self.trilogy_procedure_untag(callable, arity, "");
        self.call_callable(callable, function, arguments)
    }

    pub(crate) fn apply_function(
        &self,
        value: PointerValue<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(value, "");
        let function = self.trilogy_function_untag(callable, "");
        self.call_callable(callable, function, &[argument])
    }

    pub(crate) fn call_continuation(
        &self,
        function: PointerValue<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) -> InstructionValue<'ctx> {
        let callable = self.trilogy_callable_untag(function, "");

        let return_to = self.allocate_value("");
        let yield_to = self.allocate_value("");

        self.trilogy_callable_return_to_into(return_to, callable);
        self.trilogy_callable_yield_to_into(yield_to, callable);

        let args = vec![
            return_to.into(),
            yield_to.into(),
            self.get_end().into(),
            argument,
            self.get_callable_closure(callable).into(),
        ];

        let function = self.trilogy_continuation_untag(callable, "");
        // NOTE: cleanup will be inserted here
        let call = self
            .builder
            .build_indirect_call(self.continuation_type(), function, &args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();
        call.try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap()
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

        let parent_closure = self.allocate_value("");
        self.trilogy_array_init_cap(parent_closure, 0, "");
        self.trilogy_callable_init_cont(
            return_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            parent_closure,
            chain_function.as_global_value().as_pointer_value(),
        );
        self.trilogy_callable_init_cont(
            yield_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            parent_closure,
            yield_function.as_global_value().as_pointer_value(),
        );
        self.trilogy_callable_init_cont(
            end_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            parent_closure,
            end_function.as_global_value().as_pointer_value(),
        );

        let call = self
            .builder
            .build_indirect_call(
                self.procedure_type(0, false),
                function,
                &[
                    return_continuation.into(),
                    yield_continuation.into(),
                    end_continuation.into(),
                    return_continuation.into(),
                ],
                "",
            )
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        self.builder.build_return(None).unwrap();

        let entry = self.context.append_basic_block(yield_function, "entry");
        self.builder.position_at_end(entry);
        let effect = self.get_continuation();
        _ = self.trilogy_unhandled_effect(effect);

        let entry = self.context.append_basic_block(end_function, "entry");
        self.builder.position_at_end(entry);
        _ = self.trilogy_execution_ended();

        let entry = self.context.append_basic_block(chain_function, "entry");
        self.builder.position_at_end(entry);
        self.get_continuation()
    }

    pub(crate) fn call_internal(
        &self,
        target: PointerValue<'ctx>,
        procedure: FunctionValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) {
        let mut args = vec![target.into()];
        args.extend_from_slice(arguments);
        self.builder.build_call(procedure, &args, "").unwrap();
    }
}
