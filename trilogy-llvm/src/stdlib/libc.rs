use inkwell::values::FunctionValue;

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn std_libc(&self) {
        // <stdlib.h>
        self.define_exit();
        // <stdio.h>
    }

    fn define_exit(&self) {
        let c_exit = self.module.add_function(
            "exit",
            self.context
                .void_type()
                .fn_type(&[self.context.i32_type().into()], false),
            None,
        );

        let tri_exit = self.add_procedure("trilogy:c::exit", 1, true);
        let basic_block = self.context.append_basic_block(tri_exit, "entry");
        self.builder.position_at_end(basic_block);

        // TODO: convert to C_INT instead of ... this
        let argument = tri_exit.get_nth_param(1).unwrap().into_pointer_value();
        let payload = self.get_payload(argument);
        let value = self
            .builder
            .build_bit_cast(payload, self.context.i64_type(), "")
            .unwrap()
            .into_int_value();
        let value = self
            .builder
            .build_int_truncate(value, self.context.i32_type(), "")
            .unwrap();
        self.builder
            .build_call(c_exit, &[value.into()], "exit")
            .unwrap();
        self.builder.build_unreachable().unwrap();
    }

    pub(crate) fn exit(&self) -> FunctionValue<'ctx> {
        if let Some(exit) = self.module.get_function("trilogy:c::exit") {
            return exit;
        }
        self.add_procedure("trilogy:c::exit", 1, true)
    }
}
