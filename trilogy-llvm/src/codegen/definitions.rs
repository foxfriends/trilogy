//! Handles creating function definitions at the LLVM level.
use crate::codegen::Codegen;
use inkwell::llvm_sys::LLVMCallConv;
use inkwell::module::Linkage;
use inkwell::values::FunctionValue;
use source_span::Span;

impl<'ctx> Codegen<'ctx> {
    /// Adds a new function to the module, to be used as a continuation function.
    ///
    /// The parameters to this continuation are, in order:
    /// 1. return_to
    /// 2. yield_to
    /// 3. end_to
    /// 4. cancel_to
    /// 5. resume_to
    /// 6. break_to
    /// 7. continue_to
    /// 8. next_to
    /// 9. done_to
    /// 10. value
    /// 11. closure
    ///
    /// Typically only `value` is provided by the caller directly. The rest are stored in the continuation
    /// object and provided by the calling convention.
    pub(crate) fn add_continuation(&self, name: &str) -> FunctionValue<'ctx> {
        let (parent_name, parent_linkage_name, span) = self.get_current_definition();
        let name = if name.is_empty() {
            parent_linkage_name
        } else {
            format!("{parent_linkage_name}.{name}")
        };
        let function =
            self.module
                .add_function(&name, self.continuation_type(1), Some(Linkage::Private));
        function.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        function.set_subprogram(self.di.create_function(
            &parent_name,
            function.get_name().to_str().unwrap(),
            self.di.continuation_di_type(),
            span,
            true,
            true,
        ));
        function.get_nth_param(0).unwrap().set_name("return_to");
        function.get_nth_param(1).unwrap().set_name("yield_to");
        function.get_nth_param(2).unwrap().set_name("end_to");
        function.get_nth_param(3).unwrap().set_name("cancel_to");
        function.get_nth_param(4).unwrap().set_name("resume_to");
        function.get_nth_param(5).unwrap().set_name("break_to");
        function.get_nth_param(6).unwrap().set_name("continue_to");
        function.get_nth_param(7).unwrap().set_name("next_to");
        function.get_nth_param(8).unwrap().set_name("done_to");
        function.get_nth_param(9).unwrap().set_name("cont_val");
        function.get_nth_param(10).unwrap().set_name("closure");
        function
    }

    /// Adds a new function to the module, to be used as a "next to" continuation.
    ///
    /// The parameters to this continuation are, in order:
    /// - 1. return_to
    /// - 2. yield_to
    /// - 3. end_to
    /// - 4. cancel_to
    /// - 5. resume_to
    /// - 6. break_to
    /// - 7. continue_to
    /// - 8. next_to
    /// - 9. done_to
    /// - 10. next_iteration
    /// - [11 + arity). arguments
    /// - [11 + arity. closure
    ///
    /// Typically only `value` is provided by the caller directly. The rest are stored in the continuation
    /// object and provided by the calling convention.
    pub(crate) fn add_next_to_continuation(&self, arity: usize, name: &str) -> FunctionValue<'ctx> {
        let (parent_name, parent_linkage_name, span) = self.get_current_definition();
        let name = if name.is_empty() {
            parent_linkage_name
        } else {
            format!("{parent_linkage_name}.{name}/{arity}")
        };
        let function = self.module.add_function(
            &name,
            self.continuation_type(arity + 1),
            Some(Linkage::Private),
        );
        function.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        function.set_subprogram(self.di.create_function(
            &parent_name,
            function.get_name().to_str().unwrap(),
            self.di.procedure_di_type(arity + 1),
            span,
            true,
            true,
        ));
        function.get_nth_param(0).unwrap().set_name("return_to");
        function.get_nth_param(1).unwrap().set_name("yield_to");
        function.get_nth_param(2).unwrap().set_name("end_to");
        function.get_nth_param(3).unwrap().set_name("cancel_to");
        function.get_nth_param(4).unwrap().set_name("resume_to");
        function.get_nth_param(5).unwrap().set_name("break_to");
        function.get_nth_param(6).unwrap().set_name("continue_to");
        function.get_nth_param(7).unwrap().set_name("next_to");
        function.get_nth_param(8).unwrap().set_name("done_to");
        function
            .get_nth_param(9)
            .unwrap()
            .set_name("next_iteration_param");
        function.get_last_param().unwrap().set_name("closure");
        function
    }

    pub(crate) fn add_procedure(
        &self,
        name: &str,
        arity: usize,
        debug_name: &str,
        span: Span,
        has_context: bool,
        is_local_to_unit: bool,
    ) -> FunctionValue<'ctx> {
        let function = self.module.add_function(
            name,
            self.procedure_type(arity, has_context),
            Some(Linkage::Private),
        );
        function.set_subprogram(self.di.create_function(
            debug_name,
            name,
            self.di.procedure_di_type(arity),
            span,
            is_local_to_unit,
            true,
        ));
        function.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        function.get_nth_param(0).unwrap().set_name("return_to");
        function.get_nth_param(1).unwrap().set_name("yield_to");
        function.get_nth_param(2).unwrap().set_name("end_to");
        function.get_nth_param(3).unwrap().set_name("cancel_to");
        function.get_nth_param(4).unwrap().set_name("resume_to");
        function.get_nth_param(5).unwrap().set_name("break_to");
        function.get_nth_param(6).unwrap().set_name("continue_to");
        function.get_nth_param(7).unwrap().set_name("next_to");
        function.get_nth_param(8).unwrap().set_name("done_to");
        if has_context {
            function.get_last_param().unwrap().set_name("closure");
        }
        function
    }

    pub(crate) fn add_function(
        &self,
        name: &str,
        debug_name: &str,
        span: Span,
        has_context: bool,
        is_local_to_unit: bool,
    ) -> FunctionValue<'ctx> {
        self.add_procedure(name, 1, debug_name, span, has_context, is_local_to_unit)
    }

    pub(crate) fn add_rule(
        &self,
        name: &str,
        arity: usize,
        debug_name: &str,
        span: Span,
        has_context: bool,
        is_local_to_unit: bool,
    ) -> FunctionValue<'ctx> {
        self.add_procedure(name, arity, debug_name, span, has_context, is_local_to_unit)
    }

    pub(crate) fn add_accessor(
        &self,
        name: &str,
        has_context: bool,
        linkage: Linkage,
    ) -> FunctionValue<'ctx> {
        let accessor =
            self.module
                .add_function(name, self.accessor_type(has_context), Some(linkage));
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        accessor.get_nth_param(0).unwrap().set_name("output");
        if has_context {
            accessor.get_nth_param(1).unwrap().set_name("closure");
        }
        accessor
    }

    /// Imports an existing accessor from another module. Imported accessors are necessarily top
    /// level, so they will never have context.
    pub(crate) fn import_accessor(&self, name: &str) -> FunctionValue<'ctx> {
        if let Some(function) = self.module.get_function(name) {
            return function;
        }
        self.add_accessor(name, false, Linkage::External)
    }

    pub(crate) fn add_external_declaration(
        &self,
        name: &str,
        arity: usize,
        span: Span,
    ) -> FunctionValue<'ctx> {
        let function =
            self.module
                .add_function(name, self.external_type(arity), Some(Linkage::External));
        function.set_subprogram(self.di.create_function(
            name,
            name,
            self.di.procedure_di_type(arity),
            span,
            false,
            false,
        ));
        function
    }
}
