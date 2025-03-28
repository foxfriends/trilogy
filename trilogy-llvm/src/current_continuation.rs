use crate::codegen::{Brancher, Codegen};
use inkwell::AddressSpace;
use inkwell::values::{BasicValue, FunctionValue, PointerValue};

enum EndType<'a, 'ctx> {
    Capture(&'a Brancher<'ctx>),
    CloseBranch(&'a Brancher<'ctx>),
    Close,
}

enum ContinuationType<'ctx> {
    Continuation,
    Resume,
    Continue(PointerValue<'ctx>),
}

impl<'ctx> Codegen<'ctx> {
    /// Constructs a TrilogyValue that represents the continuation from a branch.
    /// This does not end the branch, only adds a capture point to it.
    pub(crate) fn capture_current_continuation(
        &self,
        continuation_function: FunctionValue<'ctx>,
        branch: &Brancher<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        self.construct_current_continuation(
            continuation_function,
            EndType::Capture(branch),
            ContinuationType::Continuation,
            name,
        )
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
        let end_type = match branch {
            Some(branch) => EndType::CloseBranch(branch),
            None => EndType::Close,
        };
        self.construct_current_continuation(
            continuation_function,
            end_type,
            ContinuationType::Continuation,
            name,
        )
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
        let break_to = self.get_break("");
        let continue_to = self.get_continue("");
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
        self.construct_current_continuation(
            continuation_function,
            EndType::Close,
            ContinuationType::Resume,
            name,
        )
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
        self.construct_current_continuation(
            continuation_function,
            EndType::Close,
            ContinuationType::Continue(break_to),
            name,
        )
    }

    /// Constructs a TrilogyValue that represents the current continuation.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    fn construct_current_continuation(
        &self,
        continuation_function: FunctionValue<'ctx>,
        capture_type: EndType<'_, 'ctx>,
        continuation_type: ContinuationType<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let cancel_to = self.get_cancel("");
        let break_to = match continuation_type {
            ContinuationType::Continue(break_to) => break_to,
            _ => self.get_break(""),
        };
        let continue_to = self.get_continue("");

        self.bind_temporary(continuation);

        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        match capture_type {
            EndType::Capture(branch) => {
                self.add_branch_capture(branch, closure.as_instruction_value().unwrap());
            }
            EndType::CloseBranch(branch) => {
                self.add_branch_end_as_close(branch, closure.as_instruction_value().unwrap());
            }
            EndType::Close => {
                self.end_continuation_point_as_close(closure.as_instruction_value().unwrap());
            }
        }
        match continuation_type {
            ContinuationType::Resume => {
                self.trilogy_callable_init_resume(
                    continuation,
                    return_to,
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                    cancel_to,
                    break_to,
                    continue_to,
                    closure,
                    continuation_function,
                );
            }
            ContinuationType::Continuation => {
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
            }
            ContinuationType::Continue(..) => {
                self.trilogy_callable_init_continue(
                    continuation,
                    return_to,
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                    cancel_to,
                    break_to,
                    continue_to,
                    closure,
                    continuation_function,
                );
            }
        }

        continuation
    }
}
