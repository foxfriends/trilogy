use crate::codegen::{Codegen, NeverValue};
use inkwell::AddressSpace;
use inkwell::builder::Builder;
use inkwell::module::Linkage;
use inkwell::types::FunctionType;
use inkwell::values::{BasicValue, FunctionValue, InstructionValue, IntValue, PointerValue};

impl<'ctx> Codegen<'ctx> {
    /// Bare functions do not satisfy any particular calling convention, and are intended
    /// for use internally, to facilitate various other combinations of instructions. These
    /// will never be exposed to Trilogy, as they reference things that don't exist in the language.
    fn declare_bare(&self, name: &str, ty: FunctionType<'ctx>) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(name) {
            return func;
        }
        self.module.add_function(name, ty, Some(Linkage::External))
    }

    #[allow(dead_code, reason = "for debugging")]
    pub(crate) fn debug_print(&self, value: impl AsRef<str>) {
        let f = self.declare_bare(
            "printf",
            self.context.i32_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                true,
            ),
        );
        let debug_str = self.module.add_global(
            self.context
                .i8_type()
                .array_type(value.as_ref().len() as u32 + 1),
            None,
            "",
        );
        debug_str.set_initializer(&self.context.const_string(value.as_ref().as_bytes(), true));
        self.builder
            .build_call(f, &[debug_str.as_pointer_value().into()], "")
            .unwrap();
    }

    /// Untags a boolean value. The return value is of type `i1`.
    pub(crate) fn trilogy_boolean_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let f = self.declare_bare(
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

    pub(crate) fn trilogy_atom_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_atom_untag",
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
        len: usize,
        string: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_string_init_new",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[
                    value.into(),
                    self.usize_type().const_int(len as u64, false).into(),
                    string.into(),
                ],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "this is a crazy C function sorry"
    )]
    pub(crate) fn trilogy_number_init_const(
        &self,
        value: PointerValue<'ctx>,
        re_is_negative: IntValue<'ctx>,
        re_numer_length: usize,
        re_numer: PointerValue<'ctx>,
        re_denom_length: usize,
        re_denom: PointerValue<'ctx>,
        im_is_negative: IntValue<'ctx>,
        im_numer_length: usize,
        im_numer: PointerValue<'ctx>,
        im_denom_length: usize,
        im_denom: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_number_init_const",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.bool_type().into(),
                    self.usize_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.bool_type().into(),
                    self.usize_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[
                    value.into(),
                    re_is_negative.into(),
                    self.usize_type()
                        .const_int(re_numer_length as u64, false)
                        .into(),
                    re_numer.into(),
                    self.usize_type()
                        .const_int(re_denom_length as u64, false)
                        .into(),
                    re_denom.into(),
                    im_is_negative.into(),
                    self.usize_type()
                        .const_int(im_numer_length as u64, false)
                        .into(),
                    im_numer.into(),
                    self.usize_type()
                        .const_int(im_denom_length as u64, false)
                        .into(),
                    im_denom.into(),
                ],
                name,
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_tuple_init_new(
        &self,
        value: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_tuple_init_new",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into(), lhs.into(), rhs.into()], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_tuple_assume(
        &self,
        t: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_tuple_assume",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[t.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_tuple_left(&self, target: PointerValue<'ctx>, t: PointerValue<'ctx>) {
        let f = self.declare_bare(
            "trilogy_tuple_left",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), t.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_tuple_right(&self, target: PointerValue<'ctx>, t: PointerValue<'ctx>) {
        let f = self.declare_bare(
            "trilogy_tuple_right",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), t.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_struct_init_new(
        &self,
        value: PointerValue<'ctx>,
        tag: IntValue<'ctx>,
        val: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_struct_init_new",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[value.into(), tag.into(), val.into()], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_bits_init_new(
        &self,
        value: PointerValue<'ctx>,
        len: IntValue<'ctx>,
        string: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_bits_init_new",
            self.context.ptr_type(AddressSpace::default()).fn_type(
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
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    /// Initializes an atom value
    pub(crate) fn trilogy_atom_init(&self, target: PointerValue<'ctx>, value: IntValue<'ctx>) {
        let f = self.declare_bare(
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
        let f = self.declare_bare(
            "trilogy_array_init_cap",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[
                    value.into(),
                    self.usize_type().const_int(cap as u64, false).into(),
                ],
                name,
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_array_assume(
        &self,
        t: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        self.trilogy_array_assume_in(&self.builder, t, name)
    }

    pub(crate) fn trilogy_array_assume_in(
        &self,
        builder: &Builder<'ctx>,
        t: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_array_assume",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        builder
            .build_call(f, &[t.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_array_len(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_array_len",
            self.usize_type().fn_type(
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

    pub(crate) fn trilogy_array_push(&self, array: PointerValue<'ctx>, value: PointerValue<'ctx>) {
        let f = self.declare_bare(
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
        let f = self.declare_bare(
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

    pub(crate) fn trilogy_array_slice(
        &self,
        output: PointerValue<'ctx>,
        array: PointerValue<'ctx>,
        start: IntValue<'ctx>,
        end: IntValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_array_slice",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                    self.usize_type().into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[output.into(), array.into(), start.into(), end.into()],
                "",
            )
            .unwrap();
    }

    pub(crate) fn trilogy_array_at(
        &self,
        output: PointerValue<'ctx>,
        array: PointerValue<'ctx>,
        index: usize,
    ) {
        self.trilogy_array_at_in_dyn(
            &self.builder,
            output,
            array,
            self.usize_type().const_int(index as u64, false),
        )
    }

    pub(crate) fn trilogy_array_at_dyn(
        &self,
        output: PointerValue<'ctx>,
        array: PointerValue<'ctx>,
        index: IntValue<'ctx>,
    ) {
        self.trilogy_array_at_in_dyn(&self.builder, output, array, index)
    }

    pub(crate) fn trilogy_array_at_in(
        &self,
        builder: &Builder<'ctx>,
        output: PointerValue<'ctx>,
        array: PointerValue<'ctx>,
        index: usize,
    ) {
        self.trilogy_array_at_in_dyn(
            builder,
            output,
            array,
            self.usize_type().const_int(index as u64, false),
        );
    }

    pub(crate) fn trilogy_array_at_in_dyn(
        &self,
        builder: &Builder<'ctx>,
        output: PointerValue<'ctx>,
        array: PointerValue<'ctx>,
        index: IntValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_array_at",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                ],
                false,
            ),
        );
        builder
            .build_call(f, &[output.into(), array.into(), index.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_record_init_cap(
        &self,
        value: PointerValue<'ctx>,
        cap: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_record_init_cap",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[
                    value.into(),
                    self.usize_type().const_int(cap as u64, false).into(),
                ],
                name,
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_record_assume(
        &self,
        t: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_record_assume",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[t.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_record_insert(
        &self,
        record: PointerValue<'ctx>,
        key: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_record_insert",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[record.into(), key.into(), value.into()], "")
            .unwrap();
    }

    /// Untags a callable value. The returned PointerValue points to a `trilogy_callable_value`.
    pub(crate) fn trilogy_callable_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
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

    /// Assumes a callable value. The returned PointerValue points to a `trilogy_callable_value`.
    pub(crate) fn trilogy_callable_assume(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_callable_assume",
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
        let f = self.declare_bare(
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

    pub(crate) fn trilogy_callable_return_to_into(
        &self,
        target: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_return_to_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), callable.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_callable_yield_to_into(
        &self,
        target: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_yield_to_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), callable.into()], "")
            .unwrap();
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn trilogy_callable_promote(
        &self,
        target: PointerValue<'ctx>,
        return_to: PointerValue<'ctx>,
        yield_to: PointerValue<'ctx>,
        cancel_to: PointerValue<'ctx>,
        resume_to: PointerValue<'ctx>,
        break_to: PointerValue<'ctx>,
        continue_to: PointerValue<'ctx>,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_promote",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
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
                    target.into(),
                    return_to.into(),
                    yield_to.into(),
                    cancel_to.into(),
                    resume_to.into(),
                    break_to.into(),
                    continue_to.into(),
                    next_to.into(),
                    done_to.into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn trilogy_callable_cancel_to_into(
        &self,
        target: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_cancel_to_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), callable.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_callable_resume_to_into(
        &self,
        target: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_resume_to_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), callable.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_callable_break_to_into(
        &self,
        target: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_break_to_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), callable.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_callable_continue_to_into(
        &self,
        target: PointerValue<'ctx>,
        callable: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_continue_to_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), callable.into()], "")
            .unwrap();
    }

    /// Untags a procedure. The value should be a `trilogy_callable_value` and the return pointer will be
    /// a bare function pointer.
    pub(crate) fn trilogy_procedure_untag(
        &self,
        value: PointerValue<'ctx>,
        arity: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
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

    /// Untags a rule. The value should be a `trilogy_callable_value` and the return pointer will be
    /// a bare function pointer.
    pub(crate) fn trilogy_rule_untag(
        &self,
        value: PointerValue<'ctx>,
        arity: usize,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_rule_untag",
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
        let f = self.declare_bare(
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
        let f = self.declare_bare(
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
        let f = self.declare_bare(
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

    pub(crate) fn trilogy_value_clone_into(
        &self,
        into: PointerValue<'ctx>,
        from: PointerValue<'ctx>,
    ) {
        self.trilogy_value_clone_into_in(&self.builder, into, from);
    }

    pub(crate) fn trilogy_value_clone_into_in(
        &self,
        builder: &Builder<'ctx>,
        into: PointerValue<'ctx>,
        from: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_value_clone_into",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        builder
            .build_call(f, &[into.into(), from.into()], "")
            .unwrap();
    }

    pub(crate) fn trilogy_value_destroy(
        &self,
        value: PointerValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        self.trilogy_value_destroy_in(&self.builder, value)
    }

    pub(crate) fn trilogy_value_destroy_in(
        &self,
        builder: &Builder<'ctx>,
        value: PointerValue<'ctx>,
    ) -> InstructionValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_value_destroy",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        builder
            .build_call(f, &[value.into()], "")
            .unwrap()
            .try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap()
    }

    pub(crate) fn trilogy_callable_init_proc(
        &self,
        t: PointerValue<'ctx>,
        arity: usize,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_proc",
            self.context.ptr_type(AddressSpace::default()).fn_type(
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
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn trilogy_callable_init_func(
        &self,
        t: PointerValue<'ctx>,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_func",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
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
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn trilogy_callable_init_fn(
        &self,
        t: PointerValue<'ctx>,
        closure: PointerValue<'ctx>,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_fn",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
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
                    closure.into(),
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn trilogy_callable_init_rule(
        &self,
        t: PointerValue<'ctx>,
        arity: usize,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_rule",
            self.context.ptr_type(AddressSpace::default()).fn_type(
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
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn trilogy_callable_init_qy(
        &self,
        t: PointerValue<'ctx>,
        arity: usize,
        closure: PointerValue<'ctx>,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_qy",
            self.context.ptr_type(AddressSpace::default()).fn_type(
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
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn exit(&self, t: PointerValue<'ctx>) -> NeverValue {
        let f = self.declare_bare(
            "exit_",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        let call = self
            .builder
            .build_call(f, &[t.into()], "")
            .unwrap()
            .try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap();
        self.end_continuation_point_as_clean(call);
        self.builder.build_unreachable().unwrap();
        NeverValue
    }

    pub(crate) fn trilogy_reference_close(&self, t: PointerValue<'ctx>) {
        self.trilogy_reference_close_in(&self.builder, t);
    }

    pub(crate) fn trilogy_reference_close_in(
        &self,
        builder: &Builder<'ctx>,
        t: PointerValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_reference_close",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        builder.build_call(f, &[t.into()], "").unwrap();
    }

    pub(crate) fn trilogy_reference_assume(&self, t: PointerValue<'ctx>) -> PointerValue<'ctx> {
        self.trilogy_reference_assume_in(&self.builder, t)
    }

    pub(crate) fn trilogy_reference_assume_in(
        &self,
        builder: &Builder<'ctx>,
        t: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_reference_assume",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        builder
            .build_call(f, &[t.into()], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_reference_init_empty(
        &self,
        t: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_reference_init_empty",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        self.builder
            .build_call(f, &[t.into()], name)
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_reference_to(
        &self,
        pointer: PointerValue<'ctx>,
        pointee: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        self.trilogy_reference_to_in(&self.builder, pointer, pointee)
    }

    pub(crate) fn trilogy_reference_to_in(
        &self,
        builder: &Builder<'ctx>,
        pointer: PointerValue<'ctx>,
        pointee: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_reference_to",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        builder
            .build_call(f, &[pointer.into(), pointee.into()], "")
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
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_do",
            self.context.ptr_type(AddressSpace::default()).fn_type(
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
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn trilogy_callable_init_cont(
        &self,
        t: PointerValue<'ctx>,
        return_to: PointerValue<'ctx>,
        yield_to: PointerValue<'ctx>,
        cancel_to: PointerValue<'ctx>,
        resume_to: PointerValue<'ctx>,
        break_to: PointerValue<'ctx>,
        continue_to: PointerValue<'ctx>,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
        closure: PointerValue<'ctx>,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_cont",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
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
                    return_to.into(),
                    yield_to.into(),
                    cancel_to.into(),
                    resume_to.into(),
                    break_to.into(),
                    continue_to.into(),
                    next_to.into(),
                    done_to.into(),
                    closure.into(),
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn trilogy_callable_init_resume(
        &self,
        t: PointerValue<'ctx>,
        return_to: PointerValue<'ctx>,
        yield_to: PointerValue<'ctx>,
        cancel_to: PointerValue<'ctx>,
        resume_to: PointerValue<'ctx>,
        break_to: PointerValue<'ctx>,
        continue_to: PointerValue<'ctx>,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
        closure: PointerValue<'ctx>,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_resume",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
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
                    return_to.into(),
                    yield_to.into(),
                    cancel_to.into(),
                    resume_to.into(),
                    break_to.into(),
                    continue_to.into(),
                    next_to.into(),
                    done_to.into(),
                    closure.into(),
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    #[expect(clippy::too_many_arguments)]
    pub(crate) fn trilogy_callable_init_continue(
        &self,
        t: PointerValue<'ctx>,
        return_to: PointerValue<'ctx>,
        yield_to: PointerValue<'ctx>,
        cancel_to: PointerValue<'ctx>,
        resume_to: PointerValue<'ctx>,
        break_to: PointerValue<'ctx>,
        continue_to: PointerValue<'ctx>,
        next_to: PointerValue<'ctx>,
        done_to: PointerValue<'ctx>,
        closure: PointerValue<'ctx>,
        function: FunctionValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_callable_init_continue",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
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
                    return_to.into(),
                    yield_to.into(),
                    cancel_to.into(),
                    resume_to.into(),
                    break_to.into(),
                    continue_to.into(),
                    next_to.into(),
                    done_to.into(),
                    closure.into(),
                    function.as_global_value().as_pointer_value().into(),
                ],
                "",
            )
            .unwrap();
    }

    pub(crate) fn trilogy_unhandled_effect(&self, effect: PointerValue<'ctx>) -> NeverValue {
        let f = self.declare_bare(
            "trilogy_unhandled_effect",
            self.context.void_type().fn_type(
                &[self.context.ptr_type(AddressSpace::default()).into()],
                false,
            ),
        );
        let call = self
            .builder
            .build_call(f, &[effect.into()], "")
            .unwrap()
            .try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap();
        self.end_continuation_point_as_clean(call);
        self.builder.build_unreachable().unwrap();
        NeverValue
    }

    pub(crate) fn trilogy_execution_ended(&self) -> NeverValue {
        let f = self.declare_bare(
            "trilogy_execution_ended",
            self.context.void_type().fn_type(&[], false),
        );
        let call = self
            .builder
            .build_call(f, &[], "")
            .unwrap()
            .try_as_basic_value()
            .either(|l| l.as_instruction_value(), Some)
            .unwrap();
        self.end_continuation_point_as_clean(call);
        self.builder.build_unreachable().unwrap();
        NeverValue
    }

    pub(crate) fn trilogy_module_untag(
        &self,
        value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_module_untag",
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

    pub(crate) fn trilogy_module_init_new(
        &self,
        target: PointerValue<'ctx>,
        len: IntValue<'ctx>,
        member_ids: PointerValue<'ctx>,
        members: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_module_init_new",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(
                f,
                &[target.into(), len.into(), member_ids.into(), members.into()],
                name,
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_module_init_new_closure(
        &self,
        target: PointerValue<'ctx>,
        len: IntValue<'ctx>,
        member_ids: PointerValue<'ctx>,
        members: PointerValue<'ctx>,
        closure: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_bare(
            "trilogy_module_init_new_closure",
            self.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.usize_type().into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
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
                    target.into(),
                    len.into(),
                    member_ids.into(),
                    members.into(),
                    closure.into(),
                ],
                name,
            )
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_pointer_value()
    }

    pub(crate) fn trilogy_module_find(
        &self,
        target: PointerValue<'ctx>,
        module: PointerValue<'ctx>,
        id: IntValue<'ctx>,
    ) {
        let f = self.declare_bare(
            "trilogy_module_find",
            self.context.void_type().fn_type(
                &[
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.ptr_type(AddressSpace::default()).into(),
                    self.context.i64_type().into(),
                ],
                false,
            ),
        );
        self.builder
            .build_call(f, &[target.into(), module.into(), id.into()], "")
            .unwrap();
    }
}
