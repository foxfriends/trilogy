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

    fn declare_free(&self) -> FunctionValue<'ctx> {
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

    fn free(&self, ptr: PointerValue<'ctx>, name: &str) {
        let free = self.declare_free();
        self.builder.build_call(free, &[ptr.into()], name).unwrap();
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
        let tri_exit = self.exit();
        let scope = Scope::begin(tri_exit);
        let basic_block = self.context.append_basic_block(tri_exit, "entry");
        self.builder.position_at_end(basic_block);

        let argument = tri_exit.get_nth_param(1).unwrap().into_pointer_value();
        let value = self.untag_integer(&scope, argument);
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

    fn to_c_str(&self, scope: &Scope<'ctx>, tri_str: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let string = self.untag_string(scope, tri_str);
        let string = self
            .builder
            .build_int_to_ptr(
                string,
                self.context.ptr_type(AddressSpace::default()),
                "t_str",
            )
            .unwrap();
        // Allocate a space for the null terminated C string
        let length = self.get_string_value_length(string, "len");
        let malloc_length = self
            .builder
            .build_int_add(
                length,
                self.context.i64_type().const_int(1, false),
                "nul_len",
            )
            .unwrap();
        let c_str = self.malloc(malloc_length, "c_str");
        let string = self.get_string_value_pointer(string, "t_str.content");

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
                    c_str.into(),
                    string.into(),
                    length.into(),
                    self.context.bool_type().const_int(1, false).into(),
                ],
                "",
            )
            .unwrap();
        let nul_ptr = unsafe {
            self.builder
                .build_gep(
                    self.context.i8_type().array_type(0),
                    c_str,
                    &[self.context.i32_type().const_zero(), length],
                    "nul_ptr",
                )
                .unwrap()
        };
        self.builder
            .build_store(nul_ptr, self.context.i8_type().const_zero())
            .unwrap();
        c_str
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

        let tri_printf = self.printf();
        let scope = Scope::begin(tri_printf);
        let basic_block = self.context.append_basic_block(tri_printf, "entry");
        self.builder.position_at_end(basic_block);

        // Extract the string payload from the parameter
        let string = tri_printf.get_nth_param(1).unwrap().into_pointer_value();
        let c_str = self.to_c_str(&scope, string);
        let error_code = self
            .builder
            .build_call(
                c_printf,
                &[safe_fmt.as_pointer_value().into(), c_str.into()],
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
        self.free(c_str, "");
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
