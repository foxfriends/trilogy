use crate::Codegen;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    debug_info::AsDIScope,
    llvm_sys::{debuginfo::LLVMDIFlagPublic, LLVMCallConv},
    module::Linkage,
    values::{BasicMetadataValueEnum, FunctionValue},
};
use source_span::Span;
use trilogy_ir::ir;

const MAIN_NAME: &str = "trilogy:::main";

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn declare_extern_procedure(
        &self,
        name: &str,
        arity: usize,
        linkage: Linkage,
        span: Span,
    ) {
        let accessor_name = format!("{}::{}", self.location, name);
        let wrapper_name = format!("{}::{}.fastcc", self.location, name);
        let original_function =
            self.module
                .add_function(name, self.external_type(arity), Some(Linkage::External));
        let procedure_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            name,
            Some(name),
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.procedure_di_type(arity),
            linkage != Linkage::External,
            false,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        original_function.set_subprogram(procedure_scope);

        // To allow callers to always use FastCC, we provide a wrapper around all extern procedures that
        // converts to CCC.
        let wrapper_function = self.module.add_function(
            &wrapper_name,
            self.procedure_type(arity, false),
            Some(Linkage::Private),
        );
        let wrapper_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            &wrapper_name,
            Some(&wrapper_name),
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.procedure_di_type(arity),
            true,
            true,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        wrapper_function.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        wrapper_function.set_subprogram(wrapper_scope);
        let wrapper_entry = self.context.append_basic_block(wrapper_function, "entry");
        self.builder.position_at_end(wrapper_entry);
        self.di.push_subprogram(wrapper_scope);
        self.di.push_block_scope(span);
        self.set_span(span);
        self.set_current_definition(wrapper_name.to_owned(), span);

        let ret_val = self.allocate_value("");
        let mut params = vec![ret_val.into()];
        params.extend(wrapper_function.get_param_iter().skip(3).map(|val| {
            let param = self.builder.build_alloca(self.value_type(), "").unwrap();
            self.builder.build_store(param, val).unwrap();
            BasicMetadataValueEnum::<'ctx>::from(param)
        }));
        self.builder
            .build_direct_call(original_function, &params, "")
            .unwrap();
        self.call_continuation(self.get_return(""), ret_val);
        self.close_continuation();
        self.di.pop_scope();
        self.di.pop_scope();

        let accessor =
            self.module
                .add_function(&accessor_name, self.accessor_type(), Some(linkage));
        self.set_current_definition(accessor_name.to_owned(), span);
        self.builder.unset_current_debug_location();
        accessor.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        self.trilogy_callable_init_proc(
            sret,
            arity,
            wrapper_function.as_global_value().as_pointer_value(),
        );
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn declare_procedure(
        &self,
        name: &str,
        arity: usize,
        linkage: Linkage,
        span: Span,
    ) -> FunctionValue<'ctx> {
        let accessor_name = format!("{}::{}", self.location, name);
        let linkage_name = if name == "main" { MAIN_NAME } else { name };

        let procedure_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            name,
            Some(linkage_name),
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.procedure_di_type(arity),
            linkage != Linkage::External,
            true,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );

        let function = self.module.add_function(
            linkage_name,
            self.procedure_type(arity, false),
            Some(Linkage::Private),
        );
        function.set_subprogram(procedure_scope);
        function.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        function.get_nth_param(0).unwrap().set_name("return_to");
        function.get_nth_param(1).unwrap().set_name("yield_to");
        function.get_nth_param(2).unwrap().set_name("end_to");

        let accessor =
            self.module
                .add_function(&accessor_name, self.accessor_type(), Some(linkage));
        accessor.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        let sret = accessor.get_nth_param(0).unwrap().into_pointer_value();
        let accessor_entry = self.context.append_basic_block(accessor, "entry");
        self.builder.position_at_end(accessor_entry);
        self.trilogy_callable_init_proc(sret, arity, function.as_global_value().as_pointer_value());
        self.builder.build_return(None).unwrap();

        function
    }

    pub(crate) fn import_procedure(
        &self,
        location: &str,
        definition: &ir::ProcedureDefinition,
    ) -> FunctionValue<'ctx> {
        let accessor_name = format!("{}::{}", location, definition.name);
        if let Some(function) = self.module.get_function(&accessor_name) {
            return function;
        }
        let accessor = self.module.add_function(
            &accessor_name,
            self.accessor_type(),
            Some(Linkage::External),
        );
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        accessor
    }

    pub(crate) fn compile_procedure(&self, definition: &ir::ProcedureDefinition) {
        assert_eq!(definition.overloads.len(), 1);
        let procedure = &definition.overloads[0];
        let name = definition.name.to_string();
        let linkage_name = if name == "main" { MAIN_NAME } else { &name };
        let function = self.module.get_function(linkage_name).unwrap();
        self.set_current_definition(linkage_name.to_owned(), procedure.span);
        self.compile_procedure_body(function, procedure);
        self.close_continuation();
    }

    pub(crate) fn compile_procedure_body(
        &self,
        function: FunctionValue<'ctx>,
        procedure: &ir::Procedure,
    ) {
        self.di.push_subprogram(function.get_subprogram().unwrap());
        self.di.push_block_scope(procedure.span);
        let entry = self.context.append_basic_block(function, "entry");

        'body: {
            self.builder.position_at_end(entry);
            self.set_span(procedure.head_span);
            for (n, param) in procedure.parameters.iter().enumerate() {
                // NOTE: params start at 3, due to return, yield, and end
                let value = function
                    .get_nth_param(n as u32 + 3)
                    .unwrap()
                    .into_struct_value();
                let value_param = self.builder.build_alloca(self.value_type(), "").unwrap();
                self.builder.build_store(value_param, value).unwrap();
                if self.compile_pattern_match(param, value_param).is_none() {
                    break 'body;
                }
            }

            if let Some(value) = self.compile_expression(&procedure.body, "") {
                // There is no implicit return of the final value of a procedure. That value is lost,
                // and unit is returned instead. It is most likely that there is a return in the body,
                // in which case we never reach this point
                self.trilogy_value_destroy(value);
                let ret = self.get_return("");
                self.call_continuation(ret, self.allocate_const(self.unit_const(), ""));
            }
        }

        self.di.pop_scope();
        self.di.pop_scope();
    }
}
