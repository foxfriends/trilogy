#![expect(dead_code, reason = "WIP")]

use inkwell::{
    module::Linkage,
    types::FunctionType,
    values::{FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    fn declare_internal(&self, name: &str, ty: FunctionType<'ctx>) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(name) {
            return func;
        }
        self.module.add_function(name, ty, Some(Linkage::External))
    }

    /// Untags a unit, though returns nothing. Fairly useless except as an assertion.
    pub(crate) fn untag_unit(&self, value: PointerValue<'ctx>, name: &str) {
        let f = self.declare_internal(
            "untag_unit",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder.build_call(f, &[value.into()], name).unwrap();
    }

    /// Untags a boolean value. The return value is of type `i1`.
    pub(crate) fn untag_boolean(&self, value: PointerValue<'ctx>, name: &str) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "untag_boolean",
            self.context.bool_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value()
    }

    /// Untags a character value. The return value is of i32 type (a unicode code point).
    pub(crate) fn untag_character(&self, value: PointerValue<'ctx>, name: &str) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "untag_character",
            self.context.i32_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value()
    }

    /// Untags an integer value.
    pub(crate) fn untag_integer(&self, value: PointerValue<'ctx>, name: &str) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "untag_integer",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value()
    }

    /// Untags a string value. The returned PointerValue points to a value of `string_value_type`.
    pub(crate) fn untag_string(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_string",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    /// Untags an atom value.
    pub(crate) fn untag_atom(&self, value: PointerValue<'ctx>, name: &str) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "untag_atom",
            self.context.i64_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value()
    }

    pub(crate) fn untag_bits(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_bits",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn untag_struct(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_struct",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn untag_tuple(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_tuple",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn untag_array(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_array",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn untag_set(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_set",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn untag_record(&self, value: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_record",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    /// Untags a callable value. The returned PointerValue points to a `trilogy_callable_value`.
    pub(crate) fn untag_callable(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_callable",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    /// Untags a procedure. The value should be a `trilogy_callable_value` and the return pointer will be
    /// a bare function pointer.
    pub(crate) fn untag_procedure(
        &self,
        value: PointerValue<'ctx>,
        arity: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_procedure",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i32_type().into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[
                    value.into(),
                    self.context
                        .i32_type()
                        .const_int(arity as u64, false)
                        .into(),
                ],
                name,
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    /// Untags a function. The value should be a `trilogy_callable_value` and the return pointer will be
    /// a bare function pointer.
    pub(crate) fn untag_function(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "untag_function",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn is_structural_eq(
        &self,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "is_structural_eq",
            self.context.bool_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[lhs.into(), rhs.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_int_value()
    }

    pub(crate) fn internal_panic(&self, message: PointerValue<'ctx>) {
        let f = self.declare_internal(
            "internal_panic",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder.build_call(f, &[message.into()], "").unwrap();
        self.builder.build_unreachable().unwrap();
    }

    pub(crate) fn trilogy_value_clone_into(
        &self,
        into: PointerValue<'ctx>,
        from: PointerValue<'ctx>,
    ) {
        let f = self.declare_internal(
            "trilogy_value_clone_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[into.into(), from.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_callable_init_proc(
        &self,
        t: PointerValue<'ctx>,
        arity: usize,
        context: PointerValue<'ctx>,
        function: PointerValue<'ctx>,
    ) {
        let f = self.declare_internal(
            "trilogy_callable_init_proc",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[
                    t.into(),
                    self.context
                        .i64_type()
                        .const_int(arity as u64, false)
                        .into(),
                    context.into(),
                    function.into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn exit(&self, t: PointerValue<'ctx>) {
        let f = self.declare_internal(
            "exit_",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder.build_call(f, &[t.into()], "").unwrap();
        self.builder.build_unreachable().unwrap();
    }
}
