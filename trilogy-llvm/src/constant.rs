use crate::{scope::Scope, Codegen};
use inkwell::module::Linkage;
use trilogy_ir::ir;

impl Codegen<'_> {
    pub(crate) fn import_constant(&self, location: &str, constant: &ir::ConstantDefinition) {
        self.add_procedure(&format!("{}::{}", location, constant.name), 0, true);
    }

    pub(crate) fn compile_constant(&self, definition: &ir::ConstantDefinition, exported: bool) {
        let global = self
            .module
            .add_global(self.value_type(), None, &definition.name.to_string());
        global.set_linkage(Linkage::Private);
        global.set_initializer(&self.value_type().const_zero());
        let function = self.add_procedure(
            &format!("{}::{}", self.location, definition.name),
            0,
            exported,
        );

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
