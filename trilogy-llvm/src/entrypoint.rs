use inkwell::{
    attributes::{Attribute, AttributeLoc},
    AddressSpace, IntPredicate,
};

use crate::{codegen::Codegen, types, TrilogyValue};

impl Codegen<'_> {
    pub(crate) fn compile_standalone(&self, entrymodule: &str, entrypoint: &str) {
        let main_wrapper =
            self.module
                .add_function("main", self.context.i32_type().fn_type(&[], false), None);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");
        let exit_unit = self.context.append_basic_block(main_wrapper, "exit_unit");
        let exit_int = self.context.append_basic_block(main_wrapper, "exit_int");

        self.builder.position_at_end(basic_block);

        // Reference main
        let main_accessor = self
            .module
            .get_function(&format!("{entrymodule}::{entrypoint}"))
            .unwrap();
        let main = self.allocate_value("main");
        self.call_internal(main, main_accessor, &[]);

        // Call main
        let output = self.allocate_value("main.out");

        // TODO: get output of this call into `output`
        self.call_procedure(main, &[]);

        // Convert return value to exit code
        let tag = self.get_tag(output);
        let is_unit = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.tag_type().const_int(types::TAG_UNIT, false),
                "",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(is_unit, exit_unit, exit_int)
            .unwrap();

        self.builder.position_at_end(exit_unit);
        self.builder
            .build_return(Some(&self.context.i32_type().const_int(0, false)))
            .unwrap();

        self.builder.position_at_end(exit_int);
        let exit_code = self.trilogy_number_untag(output, "");
        let exit_code = self
            .builder
            .build_int_truncate(exit_code, self.context.i32_type(), "")
            .unwrap();
        self.builder.build_return(Some(&exit_code)).unwrap();
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
        // TODO: get output of this call into `output`
        self.call_procedure(main, &[]);
        self.builder.build_return(None).unwrap();
    }
}
