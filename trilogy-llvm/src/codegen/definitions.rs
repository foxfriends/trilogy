//! Handles creating function definitions at the LLVM level.
use crate::codegen::Codegen;
use inkwell::attributes::{Attribute, AttributeLoc};
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
    /// 5. cancel_to
    /// 6. resume_to
    /// 4. value
    /// 7. closure
    ///
    /// Typically only `value` is provided by the caller directly. The rest are stored in the continuation
    /// object and provided by the calling convention.
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

        function.set_subprogram(self.di.create_function(
            &name,
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
        function.get_nth_param(5).unwrap().set_name("cont_val");
        function.get_nth_param(6).unwrap().set_name("closure");
        function
    }

    pub(crate) fn add_procedure(
        &self,
        name: &str,
        arity: usize,
        debug_name: &str,
        span: Span,
        is_local_to_unit: bool,
    ) -> FunctionValue<'ctx> {
        let function = self.module.add_function(
            name,
            self.procedure_type(arity, false),
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
        function
    }

    pub(crate) fn add_accessor(&self, name: &str, linkage: Linkage) -> FunctionValue<'ctx> {
        let accessor = self
            .module
            .add_function(name, self.accessor_type(), Some(linkage));
        accessor.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        accessor
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
