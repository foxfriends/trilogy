use crate::codegen::{Brancher, Codegen};
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
        self.construct_current_continuation(
            continuation_function,
            Some(branch),
            self.get_break(""),
            false,
            name,
        )
    }

    /// Constructs a TrilogyValue that represents the current continuation.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    pub(crate) fn close_current_continuation(
        &self,
        continuation_function: FunctionValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        self.construct_current_continuation(
            continuation_function,
            None,
            self.get_break(""),
            false,
            name,
        )
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
            None,
            self.get_break(""),
            true,
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
        self.construct_current_continuation(continuation_function, None, break_to, true, name)
    }

    /// Constructs a TrilogyValue that represents the current continuation.
    ///
    /// This invalidates the current continuation point, all variables will be destroyed afterwards,
    /// so may not be referenced.
    fn construct_current_continuation(
        &self,
        continuation_function: FunctionValue<'ctx>,
        branch: Option<&Brancher<'ctx>>,
        break_to: PointerValue<'ctx>,
        as_resume: bool,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        let cancel_to = self.get_cancel("");
        let continue_to = self.get_continue("");

        self.bind_temporary(continuation);

        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        if let Some(branch) = branch {
            self.add_branch_capture(branch, closure.as_instruction_value().unwrap());
        } else {
            // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
            self.end_continuation_point_as_close(closure.as_instruction_value().unwrap());
        }
        if as_resume {
            self.trilogy_callable_init_resume(
                continuation,
                return_to,
                yield_to,
                cancel_to,
                break_to,
                continue_to,
                closure,
                continuation_function,
            );
        } else {
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
        }

        continuation
    }
}
