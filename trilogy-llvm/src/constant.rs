use crate::Codegen;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    debug_info::AsDIScope,
    llvm_sys::{debuginfo::LLVMDIFlagPublic, LLVMCallConv},
    module::Linkage,
    values::FunctionValue,
};
use source_span::Span;
use trilogy_ir::ir;

impl<'ctx> Codegen<'ctx> {
    fn add_constant(&self, location: &str, name: &str, linkage: Linkage) -> FunctionValue<'ctx> {
        let accessor = self.module.add_function(
            &format!("{}::{}", location, name),
            self.procedure_type(0, false),
            Some(linkage),
        );
        accessor.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        accessor.get_nth_param(0).unwrap().set_name("sretptr");
        accessor.set_call_conventions(LLVMCallConv::LLVMFastCallConv as u32);
        accessor
    }

    pub(crate) fn import_constant(&self, location: &str, constant: &ir::ConstantDefinition) {
        self.add_constant(location, &constant.name.to_string(), Linkage::External);
    }

    pub(crate) fn declare_constant(
        &self,
        constant: &ir::ConstantDefinition,
        exported: bool,
        span: Span,
    ) {
        let linkage = if exported {
            Linkage::External
        } else {
            Linkage::Private
        };
        let accessor = self.add_constant(&self.location, &constant.name.to_string(), linkage);
        let subprogram = self.di.builder.create_function(
            self.di.unit.as_debug_info_scope(),
            &constant.name.to_string(),
            None,
            self.di.unit.get_file(),
            span.start().line as u32 + 1,
            self.di.procedure_di_type(0),
            linkage != Linkage::External,
            true,
            span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        accessor.set_subprogram(subprogram);
    }

    pub(crate) fn compile_constant(&self, definition: &ir::ConstantDefinition) {
        let global = self
            .module
            .add_global(self.value_type(), None, &definition.name.to_string());
        global.set_linkage(Linkage::Private);
        global.set_initializer(&self.value_type().const_zero());

        let function = self
            .module
            .get_function(&format!("{}::{}", self.location, definition.name))
            .unwrap();

        self.di.validate();
        self.di.push_subprogram(function.get_subprogram().unwrap());
        self.set_span(definition.value.span);
        let basic_block = self.context.append_basic_block(function, "entry");
        let initialize = self.context.append_basic_block(function, "initialize");
        let initialized = self.context.append_basic_block(function, "initialized");
        self.builder.position_at_end(basic_block);

        self.branch_undefined(global.as_pointer_value(), initialize, initialized);

        self.builder.position_at_end(initialize);
        if let Some(result) = self.compile_expression(&definition.value, "") {
            self.builder
                .build_store(global.as_pointer_value(), result)
                .unwrap();
            self.builder
                .build_unconditional_branch(initialized)
                .unwrap();
        }

        // TODO: someday constants... should be constant. And deterministic
        self.builder.position_at_end(initialized);
        let return_cont = function.get_first_param().unwrap().into_pointer_value();
        self.call_continuation(return_cont, global.as_pointer_value().into());
        self.di.pop_scope();
        self.di.validate();
    }
}
