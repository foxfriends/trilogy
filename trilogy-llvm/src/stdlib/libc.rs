use crate::{codegen::Codegen, scope::Scope};
use inkwell::{intrinsics::Intrinsic, values::FunctionValue, AddressSpace};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn std_libc(&self) {
        // <stdlib.h>
        self.define_exit();
        // <stdio.h>
        self.define_printf();
    }

    fn malloc(&self) -> FunctionValue<'ctx> {
        if let Some(malloc) = self.module.get_function("malloc") {
            return malloc;
        }
        self.module.add_function(
            "malloc",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self
                    .context
                    .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
                    .into()],
                false,
            ),
            None,
        )
    }

    fn free(&self) -> FunctionValue<'ctx> {
        if let Some(free) = self.module.get_function("free") {
            return free;
        }
        self.module.add_function(
            "free",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
            None,
        )
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

    fn define_printf(&self) {
        let c_printf = self.module.add_function(
            "printf",
            self.context.i32_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                true,
            ),
            None,
        );

        let tri_printf = self.add_procedure("trilogy:c::printf", 1, true);
        let scope = Scope::begin(tri_printf);
        let basic_block = self.context.append_basic_block(tri_printf, "entry");
        self.builder.position_at_end(basic_block);

        let format = tri_printf.get_nth_param(1).unwrap().into_pointer_value();
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
        let format_ptr = self
            .builder
            .build_call(malloc, &[length.into()], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value();
        let format = self
            .builder
            .build_struct_gep(self.string_value_type(), format, 1, "")
            .unwrap();

        let memcpy = Intrinsic::find("llvm.memcpy").unwrap();
        let memcpy = memcpy
            .get_declaration(
                &self.module,
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                    self.context.bool_type().into(),
                ],
            )
            .unwrap();
        self.builder
            .build_call(
                memcpy,
                &[
                    format_ptr.into(),
                    format.into(),
                    length.into(),
                    self.context.bool_type().const_zero().into(),
                ],
                "",
            )
            .unwrap();
        let last_byte = self
            .builder
            .build_int_sub(
                length.into_int_value(),
                self.context.i64_type().const_int(1, false),
                "",
            )
            .unwrap();
        let last_byte = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type().array_type(0),
                    format_ptr,
                    &[last_byte],
                    "",
                )
                .unwrap()
        };
        self.builder
            .build_store(last_byte, self.context.i8_type().const_zero())
            .unwrap();

        let error_code = self
            .builder
            .build_call(c_printf, &[format.into()], "printf")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
        let error_code = self
            .builder
            .build_int_s_extend(error_code, self.payload_type(), "")
            .unwrap();
        self.builder
            .build_call(self.free(), &[format.into()], "")
            .unwrap();
        let error_code = self.int_value(error_code);
        self.builder.build_store(scope.sret(), error_code).unwrap();
        self.builder.build_return(None).unwrap();
    }
}
