use crate::codegen::{Brancher, Codegen};
use inkwell::AddressSpace;
use inkwell::values::{BasicValue, FunctionValue, PointerValue};

impl<'ctx> Codegen<'ctx> {
    /// Constructs a TrilogyValue that represents the continuation from a branch.
    /// This does not end the branch, only adds a capture point to it.
    pub(crate) fn capture_current_continuation(
        &self,
        continuation_function: FunctionValue<'ctx>,
        branch: &Brancher<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let cancel_to = self.get_cancel("");
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");

        self.bind_temporary(continuation);

        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.add_branch_capture(branch, closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            cancel_to,
            break_to,
            continue_to,
            closure,
            continuation_function,
        );

        continuation
    }

    /// Constructs a TrilogyValue that represents the current continuation to be used as `cancel`.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    pub(crate) fn close_current_continuation_as_cancel(
        &self,
        continuation_function: FunctionValue<'ctx>,
        branch: Option<&Brancher<'ctx>>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let cancel_to = self.get_cancel("");
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");

        self.bind_temporary(continuation);

        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        match branch {
            Some(branch) => {
                self.add_branch_end_as_close(branch, closure.as_instruction_value().unwrap());
            }
            None => {
                self.end_continuation_point_as_close(closure.as_instruction_value().unwrap());
            }
        }
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            yield_to,
            cancel_to,
            break_to,
            continue_to,
            closure,
            continuation_function,
        );

        continuation
    }

    pub(crate) fn capture_current_continuation_as_cancel(
        &self,
        continuation_function: FunctionValue<'ctx>,
        brancher: &Brancher<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let cancel_to = self.get_cancel("");
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");
        self.bind_temporary(continuation);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        self.add_branch_capture(brancher, closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            yield_to,
            cancel_to,
            break_to,
            continue_to,
            closure,
            continuation_function,
        );
        continuation
    }

    pub(crate) fn capture_current_continuation_as_yield(
        &self,
        continuation_function: FunctionValue<'ctx>,
        brancher: &Brancher<'ctx>,
        cancel_to: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let handler = self.allocate_value(name);
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_HANDLER_CLOSURE")
            .unwrap();
        self.add_branch_capture(brancher, closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            handler,
            return_to,
            yield_to,
            cancel_to,
            break_to,
            continue_to,
            closure,
            continuation_function,
        );
        handler
    }

    /// Constructs a TrilogyValue that represents the current continuation to be used as `return`.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    pub(crate) fn close_current_continuation_as_return(
        &self,
        continuation_function: FunctionValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        self.bind_temporary(continuation);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        self.end_continuation_point_as_close(closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            closure,
            continuation_function,
        );
        continuation
    }

    /// Constructs a TrilogyValue that represents the current continuation to be used as `break`.
    pub(crate) fn capture_current_continuation_as_break(
        &self,
        continuation_function: FunctionValue<'ctx>,
        brancher: &Brancher<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_HANDLER_CLOSURE")
            .unwrap();
        self.add_branch_capture(brancher, closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            break_to,
            continue_to,
            closure,
            continuation_function,
        );
        continuation
    }

    /// Constructs a TrilogyValue that represents the current continuation, marked to be called using "resume" calling convention.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    pub(crate) fn close_current_continuation_as_resume(
        &self,
        continuation_function: FunctionValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");
        self.bind_temporary(continuation);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.end_continuation_point_as_close(closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_resume(
            continuation,
            return_to,
            yield_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            break_to,
            continue_to,
            closure,
            continuation_function,
        );

        continuation
    }

    /// Constructs a TrilogyValue that represents the current continuation, marked to be called using "continue" calling convention.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    pub(crate) fn close_current_continuation_as_continue(
        &self,
        continuation_function: FunctionValue<'ctx>,
        break_to: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        self.bind_temporary(continuation);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.end_continuation_point_as_close(closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_continue(
            continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            break_to,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            closure,
            continuation_function,
        );
        continuation
    }
}
