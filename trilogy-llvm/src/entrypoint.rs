use inkwell::{
    attributes::{Attribute, AttributeLoc},
    AddressSpace,
};

use crate::{codegen::Codegen, TrilogyValue};

impl Codegen<'_> {
    pub(crate) fn compile_standalone(&self, entrymodule: &str, entrypoint: &str) {
        let main_wrapper =
            self.module
                .add_function("main", self.context.void_type().fn_type(&[], false), None);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");

        self.builder.position_at_end(basic_block);

        // Reference main
        let main_accessor = self
            .module
            .get_function(&format!("{entrymodule}::{entrypoint}"))
            .unwrap();
        let main = self.allocate_value("main");
        self.call_internal(main, main_accessor, &[]);

        // Call main
        let output = self.call_main(main);
        _ = self.exit(output);
    }

    pub(crate) fn compile_embedded(
        &self,
        entrymodule: &str,
        entrypoint: &str,
        output: *mut TrilogyValue,
    ) {
        let output_ptr = self.module.add_global(self.value_type(), None, "output");
        self.execution_engine
            .add_global_mapping(&output_ptr, output as usize);
        let main_wrapper = self.module.add_function(
            "main",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            None,
        );
        main_wrapper.add_attribute(
            AttributeLoc::Function,
            self.context
                .create_enum_attribute(Attribute::get_named_enum_kind_id("naked"), 1),
        );
        main_wrapper.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );

        let basic_block = self.context.append_basic_block(main_wrapper, "entry");

        self.builder.position_at_end(basic_block);
        // Reference main
        let main_accessor = self
            .module
            .get_function(&format!("{entrymodule}::{entrypoint}"))
            .unwrap();
        let main = self.allocate_value("main");
        self.call_internal(main, main_accessor, &[]);

        // Call main
        let return_value = self.call_main(main);
        self.builder
            .build_store(output_ptr.as_pointer_value(), return_value)
            .unwrap();
        self.builder.build_return(None).unwrap();
    }
}
