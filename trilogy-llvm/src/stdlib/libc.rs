use crate::{codegen::Codegen, scope::Scope};
use inkwell::{
    intrinsics::Intrinsic,
    values::{FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn std_libc(&self) {
        // <stdlib.h>
        self.define_exit();
        // <stdio.h>
        self.define_printf();
    }

    pub(crate) fn import_libc(&self) {
        self.exit();
        self.printf();
    }

    fn declare_malloc(&self) -> FunctionValue<'ctx> {
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

    fn malloc(&self, length: IntValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let malloc = self.declare_malloc();
        self.builder
            .build_call(malloc, &[length.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
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

    pub(crate) fn c_exit(&self) -> FunctionValue<'ctx> {
        if let Some(exit) = self.module.get_function("exit") {
            return exit;
        }
        self.module.add_function(
            "exit",
            self.context
                .void_type()
                .fn_type(&[self.context.i32_type().into()], false),
            None,
        )
    }

    fn define_exit(&self) {
        let c_exit = self.c_exit();
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
        let safe_fmt =
            self.module
                .add_global(self.context.i8_type().array_type(3), None, "safe_fmt");
        safe_fmt.set_initializer(&self.context.const_string(b"%s", true));
        safe_fmt.set_constant(true);
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

        // Extract the string payload from the parameter
        let string = tri_printf.get_nth_param(1).unwrap().into_pointer_value();
        let string = self.get_payload(string);
        let string = self
            .builder
            .build_int_to_ptr(
                string,
                self.context.ptr_type(AddressSpace::default()),
                "string",
            )
            .unwrap();
        // Allocate a space for the null terminated C string
        let length = self.get_string_value_length(string, "length");
        let malloc_length = self
            .builder
            .build_int_add(
                length,
                self.context.i64_type().const_int(1, false),
                "byte_len",
            )
            .unwrap();
        let format_ptr = self.malloc(malloc_length, "format_str");
        let string = self.get_string_value_pointer(string, "string");

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
                    string.into(),
                    length.into(),
                    self.context.bool_type().const_zero().into(),
                ],
                "",
            )
            .unwrap();
        let last_byte = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type().array_type(0),
                    format_ptr,
                    &[length],
                    "",
                )
                .unwrap()
        };
        self.builder
            .build_store(last_byte, self.context.i8_type().const_zero())
            .unwrap();

        let error_code = self
            .builder
            .build_call(
                c_printf,
                &[safe_fmt.as_pointer_value().into(), format_ptr.into()],
                "printf",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value();
        let error_code = self
            .builder
            .build_int_s_extend(error_code, self.payload_type(), "")
            .unwrap();
        self.builder
            .build_call(self.free(), &[string.into()], "")
            .unwrap();
        let error_code = self.int_value(error_code);
        self.builder.build_store(scope.sret(), error_code).unwrap();
        self.builder.build_return(None).unwrap();
    }

    pub(crate) fn printf(&self) -> FunctionValue<'ctx> {
        if let Some(printf) = self.module.get_function("trilogy:c::printf") {
            return printf;
        }
        self.add_procedure("trilogy:c::printf", 1, true)
    }
}
