use crate::codegen::{Codegen, ContinuationPoint};
use inkwell::AddressSpace;
use inkwell::values::{BasicValue, FunctionValue, PointerValue};
use std::rc::Rc;

impl<'ctx> Codegen<'ctx> {
    /// Constructs a TrilogyValue that represents the current continuation.
    ///
    /// This is typically used for internal control flow operations. The resulting continuation
    /// should not be reified to a runtime Trilogy value, as it does not preserve any runtime
    /// control flow.
    ///
    /// After this call, the current continuation point refers to the current continuation,
    /// while the continuation point of the captured continuation is returned.
    pub(crate) fn capture_current_continuation(
        &self,
        continuation_function: FunctionValue<'ctx>,
        name: &str,
    ) -> (PointerValue<'ctx>, Rc<ContinuationPoint<'ctx>>) {
        let continuation = self.allocate_value(name);
        self.bind_temporary(continuation);

        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        let shadow = self.shadow_continuation_point();
        let capture = self.capture_contination_point(closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            self.context.ptr_type(AddressSpace::default()).const_null(),
            self.context.ptr_type(AddressSpace::default()).const_null(),
            closure,
            continuation_function,
        );
        self.become_continuation_point(shadow);

        (continuation, capture)
    }

    pub(crate) fn close_current_continuation(
        &self,
        continuation_function: FunctionValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        self.bind_temporary(continuation);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        // NOTE: cleanup will be inserted here, so variables and such are invalid afterwards
        self.end_continuation_point_as_close(closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            yield_to,
            closure,
            continuation_function,
        );

        continuation
    }

    pub(crate) fn capture_current_continuation_full(
        &self,
        continuation_function: FunctionValue<'ctx>,
        name: &str,
    ) -> (PointerValue<'ctx>, Rc<ContinuationPoint<'ctx>>) {
        let continuation = self.allocate_value(name);
        let return_to = self.get_return("");
        let yield_to = self.get_yield("");
        self.bind_temporary(continuation);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();
        let shadow = self.shadow_continuation_point();
        let capture = self.capture_contination_point(closure.as_instruction_value().unwrap());
        self.trilogy_callable_init_cont(
            continuation,
            return_to,
            yield_to,
            closure,
            continuation_function,
        );
        self.become_continuation_point(shadow);
        (continuation, capture)
    }
}
