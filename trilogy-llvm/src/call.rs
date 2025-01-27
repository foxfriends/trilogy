use crate::codegen::Codegen;
use inkwell::{
    llvm_sys::LLVMCallConv,
    values::{
        BasicMetadataValueEnum, BasicValue, FunctionValue, InstructionValue, IntValue,
        LLVMTailCallKind, PointerValue,
    },
    IntPredicate,
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

    pub(crate) fn call_procedure(
        &self,
        procedure: PointerValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) {
        let callable = self.trilogy_callable_untag(procedure, "");

        let mut args = vec![
            self.get_return().into(),
            self.get_yield().into(),
            self.get_end().into(),
            todo!("build continuation"),
        ];
        args.extend_from_slice(arguments);

        let closure = self.get_callable_closure(callable, "");
        let has_closure = self.is_closure(closure);

        let direct_block = self
            .context
            .append_basic_block(self.get_function(), "call.proc");
        let closure_block = self
            .context
            .append_basic_block(self.get_function(), "call.do");
        let cont_block = self
            .context
            .append_basic_block(self.get_function(), "call.cont");

        let function = self.trilogy_procedure_untag(callable, args.len(), "");
        self.builder
            .build_conditional_branch(has_closure, closure_block, direct_block)
            .unwrap();

        self.builder.position_at_end(direct_block);
        let call = self
            .builder
            .build_indirect_call(
                self.procedure_type(args.len() - 1, false),
                function,
                &args,
                "",
            )
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(closure_block);
        args.push(closure.into());
        let call = self
            .builder
            .build_indirect_call(
                self.procedure_type(args.len() - 1, true),
                function,
                &args,
                "",
            )
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(cont_block);
    }

    pub(crate) fn call_continuation(
        &self,
        function: PointerValue<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) -> InstructionValue<'ctx> {
        let callable = self.trilogy_callable_untag(function, "");

        let args = vec![
            self.get_callable_return_to(callable, "").into(),
            self.get_callable_yield_to(callable, "").into(),
            self.get_end().into(),
            argument,
            self.get_callable_closure(callable, "").into(),
        ];

        let function = self.trilogy_continuation_untag(callable, "");
        let call = self
            .builder
            .build_indirect_call(self.continuation_type(), function, &args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        call.set_tail_call_kind(LLVMTailCallKind::LLVMTailCallKindMustTail);
        self.builder.build_return(None).unwrap();
        call.try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap()
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

    pub(crate) fn apply_function(
        &self,
        function: PointerValue<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) {
        let callable = self.trilogy_callable_untag(function, "");

        let mut args = vec![
            self.get_return().into(),
            self.get_yield().into(),
            self.get_end().into(),
            todo!("build continuation"),
            argument,
        ];

        let closure = self.get_callable_closure(callable, "");
        let has_closure = self.is_closure(closure);

        let direct_block = self
            .context
            .append_basic_block(self.get_function(), "call.func");
        let closure_block = self
            .context
            .append_basic_block(self.get_function(), "call.fn");
        let cont_block = self
            .context
            .append_basic_block(self.get_function(), "call.cont");

        let function = self.trilogy_function_untag(callable, "");
        self.builder
            .build_conditional_branch(has_closure, closure_block, direct_block)
            .unwrap();

        self.builder.position_at_end(direct_block);
        self.builder
            .build_indirect_call(self.function_type(false), function, &args, "")
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(closure_block);
        args.push(closure.into());
        let call = self
            .builder
            .build_indirect_call(self.function_type(true), function, &args, "")
            .unwrap();
        call.set_call_convention(LLVMCallConv::LLVMFastCallConv as u32);
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(cont_block);
    }
}
