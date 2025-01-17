use crate::{scope::Scope, Codegen};
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    module::Linkage,
    values::FunctionValue,
};
use trilogy_ir::ir;

impl<'ctx> Codegen<'ctx> {
    fn add_constant(&self, location: &str, name: &str, linkage: Linkage) -> FunctionValue<'ctx> {
        let procedure = self.module.add_function(
            &format!("{}::{}", location, name),
            self.procedure_type(0),
            Some(linkage),
        );
        procedure.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        procedure.get_nth_param(0).unwrap().set_name("sretptr");
        procedure
    }

    pub(crate) fn import_constant(&self, location: &str, constant: &ir::ConstantDefinition) {
        self.add_constant(location, &constant.name.to_string(), Linkage::External);
    }

    pub(crate) fn declare_constant(&self, constant: &ir::ConstantDefinition, exported: bool) {
        self.add_constant(
            &self.location,
            &constant.name.to_string(),
            if exported {
                Linkage::External
            } else {
                Linkage::Private
            },
        );
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

        let mut scope = Scope::begin(function);
        let basic_block = self.context.append_basic_block(function, "entry");
        let initialize = self.context.append_basic_block(function, "initialize");
        let initialized = self.context.append_basic_block(function, "initialized");
        self.builder.position_at_end(basic_block);

        self.branch_undefined(global.as_pointer_value(), initialize, initialized);

        self.builder.position_at_end(initialize);
        let computed = self.compile_expression(&mut scope, &definition.value);
        let computed = self
            .builder
            .build_load(self.value_type(), computed, "initial_value")
            .unwrap();
        self.builder
            .build_store(global.as_pointer_value(), computed)
            .unwrap();
        self.builder
            .build_unconditional_branch(initialized)
            .unwrap();

        self.builder.position_at_end(initialized);
        let value = self
            .builder
            .build_load(self.value_type(), global.as_pointer_value(), "")
            .unwrap();
        self.builder.build_store(scope.sret(), value).unwrap();
        self.builder.build_return(None).unwrap();
    }
}
