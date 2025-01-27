use inkwell::{
    module::Linkage,
    types::FunctionType,
    values::{FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use crate::codegen::{Codegen, NeverValue};

impl<'ctx> Codegen<'ctx> {
    fn declare_internal(&self, name: &str, ty: FunctionType<'ctx>) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(name) {
            return func;
        }
        self.module.add_function(name, ty, Some(Linkage::External))
    }

    /// Untags a boolean value. The return value is of type `i1`.
    pub(crate) fn trilogy_boolean_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_boolean_untag",
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

    /// Untags an integer value.
    pub(crate) fn trilogy_number_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_number_untag",
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

    pub(crate) fn trilogy_string_init_new(
        &self,
        value: PointerValue<'ctx>,
        len: IntValue<'ctx>,
        string: PointerValue<'ctx>,
    ) {
        let f = self.declare_internal(
            "trilogy_string_init_new",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into(), len.into(), string.into()], "")
            .unwrap();
    }

    /// Initializes an atom value
    pub(crate) fn trilogy_atom_init(&self, target: PointerValue<'ctx>, value: IntValue<'ctx>) {
        let f = self.declare_internal(
            "trilogy_atom_init",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), value.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_array_init_cap(
        &self,
        value: PointerValue<'ctx>,
        cap: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_array_init_cap",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[
                    value.into(),
                    self.context.i64_type().const_int(cap as u64, false).into(),
                ],
                name,
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_array_push(&self, array: PointerValue<'ctx>, value: PointerValue<'ctx>) {
        let f = self.declare_internal(
            "trilogy_array_push",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[array.into(), value.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_array_append(
        &self,
        array: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) {
        let f = self.declare_internal(
            "trilogy_array_append",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[array.into(), value.into()], "")
            .unwrap();
    }

    /// Untags a callable value. The returned PointerValue points to a `trilogy_callable_value`.
    pub(crate) fn trilogy_callable_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_callable_untag",
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

    pub(crate) fn trilogy_callable_closure_into(
        &self,
        target: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_callable_closure_into",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), callable.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    /// Untags a procedure. The value should be a `trilogy_callable_value` and the return pointer will be
    /// a bare function pointer.
    pub(crate) fn trilogy_procedure_untag(
        &self,
        value: PointerValue<'ctx>,
        arity: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_procedure_untag",
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

    /// Untags a continuation. The value should be a `trilogy_callable_value` and the return pointer will be
    /// a bare function pointer.
    pub(crate) fn trilogy_continuation_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_continuation_untag",
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

    /// Untags a function. The value should be a `trilogy_callable_value` and the return pointer will be
    /// a bare function pointer.
    pub(crate) fn trilogy_function_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_function_untag",
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

    pub(crate) fn trilogy_value_structural_eq(
        &self,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_value_structural_eq",
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

    pub(crate) fn internal_panic(&self, message: PointerValue<'ctx>) -> NeverValue {
        let f = self.declare_internal(
            "internal_panic",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder.build_call(f, &[message.into()], "").unwrap();
        self.builder.build_unreachable().unwrap();
        NeverValue
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

    pub(crate) fn trilogy_value_destroy(&self, value: PointerValue<'ctx>) {
        let f = self.declare_internal(
            "trilogy_value_destroy",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder.build_call(f, &[value.into()], "").unwrap();
    }

    pub(crate) fn trilogy_callable_init_proc(
        &self,
        t: PointerValue<'ctx>,
        arity: usize,
        function: PointerValue<'ctx>,
    ) {
        let f = self.declare_internal(
            "trilogy_callable_init_proc",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
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
                    function.into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn exit(&self, t: PointerValue<'ctx>) -> NeverValue {
        let f = self.declare_internal(
            "exit_",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder.build_call(f, &[t.into()], "").unwrap();
        self.builder.build_unreachable().unwrap();
        NeverValue
    }

    pub(crate) fn trilogy_reference_close(&self, t: PointerValue<'ctx>) {
        let f = self.declare_internal(
            "trilogy_reference_close",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder.build_call(f, &[t.into()], "").unwrap();
    }

    pub(crate) fn trilogy_reference_assume(&self, t: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_reference_assume",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[t.into()], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_reference_to(
        &self,
        t: PointerValue<'ctx>,
        p: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let f = self.declare_internal(
            "trilogy_reference_to",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[t.into(), p.into()], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_callable_init_do(
        &self,
        t: PointerValue<'ctx>,
        arity: usize,
        closure: PointerValue<'ctx>,
        function: PointerValue<'ctx>,
    ) {
        let f = self.declare_internal(
            "trilogy_callable_init_do",
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
                    closure.into(),
                    function.into(),
                ],
                "",
            )
            .unwrap();
    }
}
