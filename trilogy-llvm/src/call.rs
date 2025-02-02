use crate::{codegen::Codegen, types::CALLABLE_CONTINUATION};
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

    pub(crate) fn add_continuation(&self, name: &str) -> FunctionValue<'ctx> {
        let (parent_name, span) = self.get_current_definition();
        let name = if name.is_empty() {
            parent_name
        } else {
            format!("{parent_name}.{name}")
        };
        let function =
            self.module
                .add_function(&name, self.continuation_type(), Some(Linkage::Private));
        let procedure_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            &name,
            Some(function.get_name().to_str().unwrap()),
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.continuation_di_type(),
            true,
            true,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        function.set_subprogram(procedure_scope);
        function.get_nth_param(0).unwrap().set_name("return_to");
        function.get_nth_param(1).unwrap().set_name("yield_to");
        function.get_nth_param(2).unwrap().set_name("end_to");
        function.get_nth_param(3).unwrap().set_name("cont_val");
        function.get_nth_param(4).unwrap().set_name("closure");
        function
    }

    fn get_callable_closure(&self, callable: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let bound_closure = self.allocate_value("");
        self.trilogy_callable_closure_into(bound_closure, callable, "");
        bound_closure
    }

    fn call_callable(
        &self,
        value: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
        function: PointerValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
    ) -> PointerValue<'ctx> {
        let bound_closure = self.get_callable_closure(callable);

        let arity = arguments.len();
        let chain_function = self.add_continuation("");

        let continuation = self.allocate_value("cont");
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");

        let mut args = vec![continuation, self.get_yield(""), self.get_end("")];
        args.extend_from_slice(arguments);

        let parent_closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.close(
            parent_closure.as_instruction_value().unwrap(),
            self.builder.get_current_debug_location().unwrap(),
        );
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            yield_to,
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
                function,
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
            .build_indirect_call(self.procedure_type(arity, true), function, &args_loaded, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindTail);
        self.builder.build_return(None).unwrap();

        let entry = self.context.append_basic_block(chain_function, "entry");
        self.builder.position_at_end(entry);
        self.transfer_debug_info(chain_function);
        self.get_continuation("")
    }

    pub(crate) fn call_procedure(
        &self,
        value: PointerValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(value, "");
        let arity = arguments.len();
        let function = self.trilogy_procedure_untag(callable, arity, "");
        self.call_callable(value, callable, function, arguments)
    }

    pub(crate) fn apply_function(
        &self,
        value: PointerValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let callable = self.trilogy_callable_untag(value, "");
        let tag_ptr = self
            .builder
            .build_struct_gep(self.callable_value_type(), callable, 1, "")
            .unwrap();
        let tag = self
            .builder
            .build_load(self.tag_type(), tag_ptr, "")
            .unwrap()
            .into_int_value();
        let is_continuation = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.context
                    .i8_type()
                    .const_int(CALLABLE_CONTINUATION, false),
                "",
            )
            .unwrap();

        let call_continuation = self.context.append_basic_block(self.get_function(), "");
        let call_function = self.context.append_basic_block(self.get_function(), "");

        self.builder
            .build_conditional_branch(is_continuation, call_continuation, call_function)
            .unwrap();

        self.builder.position_at_end(call_continuation);
        self.call_continuation(value, argument);

        self.builder.position_at_end(call_function);
        let function = self.trilogy_function_untag(callable, "");
        self.call_callable(value, callable, function, &[argument])
    }

    pub(crate) fn call_continuation(
        &self,
        function: PointerValue<'ctx>,
        argument: PointerValue<'ctx>,
    ) {
        let callable = self.trilogy_callable_untag(function, "");
        let continuation = self.trilogy_continuation_untag(callable, "continue");

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
        self.clean(call, self.builder.get_current_debug_location().unwrap());
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
            chain_function.as_global_value().as_pointer_value(),
        );
        self.trilogy_callable_init_cont(
            yield_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            yield_closure,
            yield_function.as_global_value().as_pointer_value(),
        );
        self.trilogy_callable_init_cont(
            end_continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            end_closure,
            end_function.as_global_value().as_pointer_value(),
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
