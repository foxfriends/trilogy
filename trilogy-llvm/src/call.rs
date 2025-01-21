use crate::{codegen::Codegen, scope::Scope};
use inkwell::{
    values::{BasicMetadataValueEnum, FunctionValue, PointerValue},
    AddressSpace, IntPredicate,
};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn call_procedure(
        &self,
        scope: &Scope<'ctx>,
        target: PointerValue<'ctx>,
        procedure: PointerValue<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) {
        let callable = self.trilogy_callable_untag(procedure, "");

        let mut args = Vec::with_capacity(arguments.len() + 2);
        args.push(target.into());
        args.extend_from_slice(arguments);
        let closure = self
            .builder
            .build_struct_gep(self.callable_value_type(), callable, 2, "")
            .unwrap();
        let closure = self
            .builder
            .build_load(self.context.ptr_type(AddressSpace::default()), closure, "")
            .unwrap()
            .into_pointer_value();

        let has_closure = self
            .builder
            .build_ptr_to_int(
                closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None),
                "",
            )
            .unwrap();
        let has_closure = self
            .builder
            .build_int_compare(
                IntPredicate::NE,
                has_closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
                    .const_zero(),
                "",
            )
            .unwrap();

        let direct_block = self.context.append_basic_block(scope.function, "call.proc");
        let closure_block = self.context.append_basic_block(scope.function, "call.do");
        let cont_block = self.context.append_basic_block(scope.function, "call.cont");

        let function = self.trilogy_procedure_untag(callable, args.len() - 1, "");
        self.builder
            .build_conditional_branch(has_closure, closure_block, direct_block)
            .unwrap();

        self.builder.position_at_end(direct_block);
        self.builder
            .build_indirect_call(
                self.procedure_type(args.len() - 1, false),
                function,
                &args,
                "",
            )
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(closure_block);
        args.push(closure.into());
        self.builder
            .build_indirect_call(
                self.procedure_type(args.len() - 1, true),
                function,
                &args,
                "",
            )
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(cont_block);
    }

    pub(crate) fn call_procedure_direct(
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
        scope: &Scope<'ctx>,
        target: PointerValue<'ctx>,
        function: PointerValue<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) {
        let callable = self.trilogy_callable_untag(function, "");

        let mut args = vec![target.into(), argument];
        let closure = self
            .builder
            .build_struct_gep(self.callable_value_type(), callable, 2, "")
            .unwrap();
        let closure = self
            .builder
            .build_load(self.context.ptr_type(AddressSpace::default()), closure, "")
            .unwrap()
            .into_pointer_value();

        let has_closure = self
            .builder
            .build_ptr_to_int(
                closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None),
                "",
            )
            .unwrap();
        let has_closure = self
            .builder
            .build_int_compare(
                IntPredicate::NE,
                has_closure,
                self.context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
                    .const_zero(),
                "",
            )
            .unwrap();

        let direct_block = self.context.append_basic_block(scope.function, "call.proc");
        let closure_block = self.context.append_basic_block(scope.function, "call.do");
        let cont_block = self.context.append_basic_block(scope.function, "call.cont");

        let function = self.trilogy_function_untag(callable, "");
        self.builder
            .build_conditional_branch(has_closure, closure_block, direct_block)
            .unwrap();

        self.builder.position_at_end(direct_block);
        self.builder
            .build_indirect_call(self.procedure_type(1, false), function, &args, "")
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(closure_block);
        args.push(closure.into());
        self.builder
            .build_indirect_call(self.procedure_type(1, true), function, &args, "")
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(cont_block);
    }
}
