use std::ops::Add;

use inkwell::{
    values::{FunctionValue, PointerValue},
    AddressSpace,
};

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn std_libc(&self) {
        // <stdlib.h>
        self.define_exit();
        self.define_malloc();
        self.define_free();
        // <stdio.h>
        self.define_printf();
    }

    fn malloc(&self) -> FunctionValue<'ctx> {
        if let Some(malloc) = self.module.get_function("malloc") {
            return malloc;
        }
        // TODO: ... what is usize today?
        self.module.add_function(
            "malloc",
            self.context
                .ptr_type(AddressSpace::default())
                .fn_type(&[self.context.i64_type().into()], false),
            None,
        );
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

    fn define_printf(&self) {
        let c_printf = self.module.add_function(
            "printf",
            self.context.i32_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                true,
            ),
            None,
        );

        let tri_printf = self.add_procedure("trilogy:c::printf", 2, true);
        let basic_block = self.context.append_basic_block(tri_printf, "entry");
        self.builder.position_at_end(basic_block);

        let format = tri_printf.get_nth_param(1).unwrap().into_pointer_value();
        let _extras = tri_printf.get_nth_param(2).unwrap().into_pointer_value();

        let format = self.get_payload(format);
        let format = self
            .builder
            .build_int_to_ptr(
                format,
                self.context.ptr_type(AddressSpace::default()),
                "format",
            )
            .unwrap();
        let malloc = self.malloc();
        let length = self
            .builder
            .build_struct_gep(self.string_value_type(), format, 0, "")
            .unwrap();
        let length = self
            .builder
            .build_load(self.context.i64_type(), length, "format_length")
            .unwrap();
        let _format_ptr = self
            .builder
            .build_call(malloc, &[length.into()], "")
            .unwrap();
        let format = self
            .builder
            .build_struct_gep(self.string_value_type(), format, 1, "")
            .unwrap();
        // TODO: memcpy from format to format_ptr
        // TODO: then ensure it's NUL-terminated
        // TODO: then sort out the varargs
        self.builder
            .build_call(c_printf, &[format.into()], "printf")
            .unwrap();
        // TODO: retrieve the return value as a Trilogy integer
        // TODO: free the malloc'ed c_string
    }

    pub(crate) fn exit(&self) -> FunctionValue<'ctx> {
        if let Some(exit) = self.module.get_function("trilogy:c::exit") {
            return exit;
        }
        self.add_procedure("trilogy:c::exit", 1, true)
    }
}
