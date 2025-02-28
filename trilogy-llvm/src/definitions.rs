use crate::codegen::Codegen;
use inkwell::{
    debug_info::AsDIScope, llvm_sys::debuginfo::LLVMDIFlagPublic, module::Linkage,
    values::FunctionValue,
};

impl<'ctx> Codegen<'ctx> {
    /// Adds a new function to the module, to be used as a continuation function.
    ///
    /// The parameters to this continuation are, in order:
    /// 1. return_to
    /// 2. yield_to
    /// 3. end_to
    /// 4. value
    /// 5. closure
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

    /// Adds a new function to the module, to be used as a handler (yield) function.
    ///
    /// The parameters to the handler are, in order:
    /// 1. return_to
    /// 2. yield_to
    /// 3. end_to
    /// 4. value
    /// 5. cancel_to
    /// 6. resume_to
    /// 7. closure
    ///
    /// Typically only `value` is provided by the caller directly. The rest are stored in the continuation
    /// object or otherwise provided by the calling convention.
    pub(crate) fn add_handler_function(&self, name: &str) -> FunctionValue<'ctx> {
        let (parent_name, span) = self.get_current_definition();
        let name = if name.is_empty() {
            parent_name
        } else {
            format!("{parent_name}.{name}.handler")
        };
        let function = self
            .module
            .add_function(&name, self.handler_type(), Some(Linkage::Private));
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
        function.get_nth_param(4).unwrap().set_name("cancel_to");
        function.get_nth_param(5).unwrap().set_name("resume_to");
        function.get_nth_param(6).unwrap().set_name("closure");
        function
    }
}
