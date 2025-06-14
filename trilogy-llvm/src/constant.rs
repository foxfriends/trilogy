use crate::Codegen;
use inkwell::debug_info::AsDIScope;
use inkwell::llvm_sys::debuginfo::LLVMDIFlagPublic;
use inkwell::module::Linkage;
use trilogy_ir::{Id, ir};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_constant(
        &self,
        definition: &ir::ConstantDefinition,
        module_context: Option<Vec<Id>>,
    ) {
        let name = definition.name.to_string();
        let accessor_name = format!("{}::{}", self.module_path(), &name);
        let accessor = self.module.get_function(&accessor_name).unwrap();

        let subprogram = self.di.builder.create_function(
            self.di.unit.as_debug_info_scope(),
            &name,
            None,
            self.di.unit.get_file(),
            definition.span().start().line as u32 + 1,
            self.di.procedure_di_type(0),
            false,
            true,
            definition.span().start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        accessor.set_subprogram(subprogram);

        let has_context = accessor.count_params() == 2;
        let storage = if has_context {
            self.allocate_value("")
        } else {
            let global = self.module.add_global(self.value_type(), None, &name);
            global.set_linkage(Linkage::Private);
            global.set_initializer(&self.value_type().const_zero());
            global.as_pointer_value()
        };

        self.set_current_definition(name, accessor_name, definition.value.span, module_context);
        self.begin_function(accessor, definition.span());
        let initialize = self.context.append_basic_block(accessor, "initialize");
        let initialized = self.context.append_basic_block(accessor, "initialized");
        self.branch_undefined(storage, initialize, initialized);

        self.builder.position_at_end(initialized);
        let sret = accessor.get_first_param().unwrap().into_pointer_value();
        self.trilogy_value_clone_into(sret, storage);
        self.builder.build_return(None).unwrap();

        self.builder.position_at_end(initialize);
        // TODO: restrict constants to actually be "constant":
        // - literals
        // - basic operators
        // - constant/function/procedure/rule references
        // - partial function applications (?)
        if let Some(result) = self.compile_expression(&definition.value, "") {
            let value = self
                .builder
                .build_load(self.value_type(), result, "")
                .unwrap();
            self.builder.build_store(storage, value).unwrap();
            self.builder
                .build_unconditional_branch(initialized)
                .unwrap();
        }

        self.end_function();
    }
}
