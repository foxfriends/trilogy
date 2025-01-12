#![expect(dead_code, reason = "WIP")]
use crate::{codegen::Codegen, scope::Scope};
use inkwell::{
    basic_block::BasicBlock,
    types::{ArrayType, FunctionType, IntType, StructType},
    values::{
        BasicMetadataValueEnum, BasicValue, FunctionValue, IntValue, PointerValue, StructValue,
    },
    AddressSpace, IntPredicate,
};

pub(crate) const TAG_UNDEFINED: u64 = 0;
pub(crate) const TAG_UNIT: u64 = 1;
pub(crate) const TAG_BOOL: u64 = 2;
pub(crate) const TAG_ATOM: u64 = 3;
pub(crate) const TAG_CHAR: u64 = 4;
pub(crate) const TAG_STRING: u64 = 5;
pub(crate) const TAG_INTEGER: u64 = 6;
pub(crate) const TAG_BITS: u64 = 7;
pub(crate) const TAG_STRUCT: u64 = 8;
pub(crate) const TAG_TUPLE: u64 = 9;
pub(crate) const TAG_ARRAY: u64 = 10;
pub(crate) const TAG_SET: u64 = 11;
pub(crate) const TAG_RECORD: u64 = 12;
pub(crate) const TAG_CALLABLE: u64 = 13;

pub(crate) trait TrilogyCallable<'ctx> {
    fn build_procedure_call(
        self,
        codegen: &Codegen<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    );

    fn build_function_call(
        self,
        codegen: &Codegen<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    );
}

impl<'ctx> TrilogyCallable<'ctx> for PointerValue<'ctx> {
    fn build_procedure_call(
        self,
        codegen: &Codegen<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) {
        codegen
            .builder
            .build_indirect_call(codegen.procedure_type(args.len() - 1), self, args, name)
            .unwrap();
    }

    fn build_function_call(
        self,
        codegen: &Codegen<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) {
        codegen
            .builder
            .build_indirect_call(codegen.procedure_type(1), self, args, name)
            .unwrap();
    }
}

impl<'ctx> TrilogyCallable<'ctx> for FunctionValue<'ctx> {
    fn build_procedure_call(
        self,
        codegen: &Codegen<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) {
        codegen.builder.build_call(self, args, name).unwrap();
    }

    fn build_function_call(
        self,
        codegen: &Codegen<'ctx>,
        args: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) {
        codegen.builder.build_call(self, args, name).unwrap();
    }
}

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn allocate_const<V: BasicValue<'ctx>>(&self, value: V) -> PointerValue<'ctx> {
        let pointer = self
            .builder
            .build_alloca(self.value_type(), "const")
            .unwrap();
        self.builder.build_store(pointer, value).unwrap();
        pointer
    }

    /// Every value in a Trilogy program is represented as an instance of this "union" struct.
    /// The first field is the tag, and the second is the value, which is often a pointer
    /// to some underlying value of which the format is known to the runtime but not the
    /// Trilogy end user.
    ///
    /// The interpretation of this value is as follows:
    /// * Tag `1` = `unit`; the value field is empty.
    /// * Tag `2` = `bool`; the value field is `0x00000001` for true and `0x00000000` for false.
    /// * Tag `3` = `atom`; the value field is opaque, and is used as a unique index for this atom. This puts an implicit limit of `u64::MAX_VALUE` possible atoms in a program (which should be more than enough).
    /// * Tag `4` = `char`; the value field is `0x0000abcd` where `0xabcd` is the Unicode code point of the character.
    /// * Tag `5` = `string`; the value field is a pointer to a struct of `{ i64 length, [i8 x length] bytes }`, which is the string encoded in UTF-8 format.
    /// * Tag `6` = `number`; the value field is a pointer to an arbitrary precision number value.
    /// * Tag `7` = `bits`; the value field is a pointer to a struct of `{ i64 length, [i1 x length] bits }` which are the literal bits.
    /// * Tag `8` = `struct`; the value field is a pointer to a struct of `{ i64 tag, ptr value }` which is the atom ID, followed by a pointer to another value.
    /// * Tag `9` = `tuple`; the value field is an array of two pointers to the two values in this tuple.
    /// * Tag `10` = `array`; the value field is a pointer to a struct of `{ i64 length, [ptr x length] items }`.
    /// * Tag `11` = `set`; the value field is a pointer to a struct of `{ i64 length, [ptr x length] items }`.
    /// * Tag `12` = `record`; the value field is a pointer to a struct of `{ i64 length, [[ptr x 2] x length] items }`.
    /// * Tag `13` = `callable`; the value field is a pointer to a function.
    pub(crate) fn value_type(&self) -> StructType<'ctx> {
        self.context
            .struct_type(&[self.tag_type().into(), self.payload_type().into()], false)
    }

    pub(crate) fn tag_type(&self) -> IntType<'ctx> {
        self.context.i8_type()
    }

    pub(crate) fn usize_type(&self) -> IntType<'ctx> {
        self.context
            .ptr_sized_int_type(self.execution_engine.get_target_data(), None)
    }

    pub(crate) fn get_tag(&self, pointer: PointerValue<'ctx>) -> IntValue<'ctx> {
        let value = self
            .builder
            .build_struct_gep(self.value_type(), pointer, 0, "tag")
            .unwrap();
        self.builder
            .build_load(self.tag_type(), value, "tag")
            .unwrap()
            .into_int_value()
    }

    pub(crate) fn tag_gep(&self, pointer: PointerValue<'ctx>) -> PointerValue<'ctx> {
        self.builder
            .build_struct_gep(self.value_type(), pointer, 0, "")
            .unwrap()
    }

    pub(crate) fn set_tag(&self, pointer: PointerValue<'ctx>, tag: u64) {
        self.builder
            .build_store(self.tag_gep(pointer), self.tag_type().const_int(tag, false))
            .unwrap();
    }

    pub(crate) fn payload_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }

    pub(crate) fn payload_gep(&self, pointer: PointerValue<'ctx>) -> PointerValue<'ctx> {
        self.builder
            .build_struct_gep(self.value_type(), pointer, 1, "payload")
            .unwrap()
    }

    pub(crate) fn get_payload(&self, pointer: PointerValue<'ctx>) -> IntValue<'ctx> {
        self.builder
            .build_load(self.payload_type(), self.payload_gep(pointer), "payload")
            .unwrap()
            .into_int_value()
    }

    pub(crate) fn set_payload(&self, pointer: PointerValue<'ctx>, value: IntValue<'ctx>) {
        self.builder
            .build_store(self.payload_gep(pointer), value)
            .unwrap();
    }

    pub(crate) fn string_value_type(&self) -> StructType<'ctx> {
        self.context.struct_type(
            &[
                self.usize_type().into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        )
    }

    pub(crate) fn get_string_value_length(
        &self,
        string_value: PointerValue<'ctx>,
        name: &str,
    ) -> IntValue<'ctx> {
        let length_field = self
            .builder
            .build_struct_gep(self.string_value_type(), string_value, 0, "")
            .unwrap();
        self.builder
            .build_load(self.context.i64_type(), length_field, name)
            .unwrap()
            .into_int_value()
    }

    pub(crate) fn get_string_value_pointer(
        &self,
        string_value: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let pointer_field = self
            .builder
            .build_struct_gep(self.string_value_type(), string_value, 1, "")
            .unwrap();
        self.builder
            .build_load(
                self.context.ptr_type(AddressSpace::default()),
                pointer_field,
                name,
            )
            .unwrap()
            .into_pointer_value()
    }

    pub(crate) fn bits_value_type(&self) -> StructType<'ctx> {
        self.context
            .struct_type(&[self.usize_type().into(), self.usize_type().into()], false)
    }

    pub(crate) fn int_value_type(&self) -> StructType<'ctx> {
        self.context
            .struct_type(&[self.usize_type().into(), self.usize_type().into()], false)
    }

    pub(crate) fn real_value_type(&self) -> StructType<'ctx> {
        self.context.struct_type(
            &[self.int_value_type().into(), self.int_value_type().into()],
            false,
        )
    }

    pub(crate) fn number_value_type(&self) -> ArrayType<'ctx> {
        self.int_value_type().array_type(4)
    }

    pub(crate) fn unit_const(&self) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_UNIT, false).into(),
            self.payload_type().const_int(0, false).into(),
        ])
    }

    pub(crate) fn bool_const(&self, value: bool) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_BOOL, false).into(),
            self.payload_type().const_int(value as u64, false).into(),
        ])
    }

    pub(crate) fn atom_const(&self, atom: String) -> StructValue<'ctx> {
        let mut atoms = self.atoms.borrow_mut();
        let next = atoms.len() as u64;
        let id = atoms.entry(atom.to_owned()).or_insert(next);
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_ATOM, false).into(),
            self.payload_type().const_int(*id, false).into(),
        ])
    }

    pub(crate) fn raw_atom_value(&self, atom: IntValue<'ctx>) -> PointerValue<'ctx> {
        let pointer = self
            .builder
            .build_alloca(self.value_type(), "atom")
            .unwrap();
        self.set_tag(pointer, TAG_ATOM);
        self.set_payload(pointer, atom);
        pointer
    }

    pub(crate) fn char_const(&self, value: char) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_CHAR, false).into(),
            self.payload_type().const_int(value as u64, false).into(),
        ])
    }

    pub(crate) fn char_value(&self, value: IntValue<'ctx>) -> PointerValue<'ctx> {
        let pointer = self
            .builder
            .build_alloca(self.value_type(), "char")
            .unwrap();
        self.set_tag(pointer, TAG_CHAR);
        let int = self
            .builder
            .build_int_z_extend(value, self.payload_type(), "")
            .unwrap();
        self.set_payload(pointer, int);
        pointer
    }

    pub(crate) fn int_const(&self, value: i64) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_INTEGER, false).into(),
            self.payload_type().const_int(value as u64, false).into(),
        ])
    }

    /// NOTE: the value *must* be a 64 bit signed integer, or this will be messed up.
    pub(crate) fn int_value(&self, value: IntValue<'ctx>) -> PointerValue<'ctx> {
        let pointer = self.builder.build_alloca(self.value_type(), "int").unwrap();
        self.set_tag(pointer, TAG_INTEGER);
        self.set_payload(pointer, value);
        pointer
    }

    pub(crate) fn string_const(&self, value: &str) -> StructValue<'ctx> {
        // SAFETY: it seems the only restriction is that this must not be called outside of a
        // function, which is not checked but we will never do it.
        let bytes = value.as_bytes();
        let string = self.module.add_global(
            self.context.i8_type().array_type(bytes.len() as u32),
            None,
            "",
        );
        string.set_initializer(&self.context.const_string(bytes, false));
        string.set_constant(true);
        let string = self.string_value_type().const_named_struct(&[
            self.context
                .i64_type()
                .const_int(value.as_bytes().len() as u64, false)
                .into(),
            string.as_pointer_value().into(),
        ]);
        let global = self.module.add_global(self.string_value_type(), None, "");
        global.set_initializer(&string);
        global.set_constant(true);
        let int = self
            .builder
            .build_ptr_to_int(global.as_pointer_value(), self.payload_type(), "")
            .unwrap();
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_STRING, false).into(),
            int.into(),
        ])
    }

    pub(crate) fn callable_value(&self, target: PointerValue<'ctx>) -> PointerValue<'ctx> {
        let pointer = self
            .builder
            .build_alloca(self.value_type(), "callable")
            .unwrap();
        self.set_tag(pointer, TAG_CALLABLE);
        let int = self
            .builder
            .build_ptr_to_int(target, self.payload_type(), "")
            .unwrap();
        self.set_payload(pointer, int);
        pointer
    }

    pub(crate) fn procedure_type(&self, arity: usize) -> FunctionType<'ctx> {
        self.context.void_type().fn_type(
            &vec![self.context.ptr_type(AddressSpace::default()).into(); arity + 1],
            false,
        )
    }

    pub(crate) fn function_type(&self) -> FunctionType<'ctx> {
        self.procedure_type(1)
    }

    pub(crate) fn call_procedure(
        &self,
        procedure: impl TrilogyCallable<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> PointerValue<'ctx> {
        let output = self
            .builder
            .build_alloca(self.value_type(), "retval")
            .unwrap();
        let mut args = vec![output.into()];
        args.extend_from_slice(arguments);
        procedure.build_procedure_call(self, &args, name);
        output
    }

    pub(crate) fn apply_function(
        &self,
        function: impl TrilogyCallable<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let output = self
            .builder
            .build_alloca(self.value_type(), "retval")
            .unwrap();
        function.build_procedure_call(self, &[output.into(), argument], name);
        output
    }

    fn untag(&self, expected: u64, scope: &Scope, value: PointerValue<'ctx>) -> IntValue<'ctx> {
        let then_block = self.context.append_basic_block(scope.function, "then");
        let else_block = self.context.append_basic_block(scope.function, "else");
        let tag = self.get_tag(value);
        let cmp = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.context.i8_type().const_int(expected, false),
                "untag",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(cmp, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(else_block);
        let exit = self.c_exit();
        self.builder
            .build_call(
                exit,
                &[self.context.i32_type().const_int(255, false).into()],
                "exit_type_error",
            )
            .unwrap();
        self.builder.build_unreachable().unwrap();

        self.builder.position_at_end(then_block);
        let then_val = self.get_payload(value);
        self.builder
            .build_bit_cast(then_val, self.context.i64_type(), "")
            .unwrap()
            .into_int_value()
    }

    /// Untags a function value. The returned PointerValue is a bare function pointer, NOT
    /// a Trilogy callable value.
    pub(crate) fn untag_function(
        &self,
        scope: &Scope<'ctx>,
        value: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let untagged = self.untag(TAG_CALLABLE, scope, value);
        self.builder
            .build_int_to_ptr(untagged, self.context.ptr_type(AddressSpace::default()), "")
            .unwrap()
    }

    /// Untags an integer value.
    pub(crate) fn untag_integer(
        &self,
        scope: &Scope<'ctx>,
        value: PointerValue<'ctx>,
    ) -> IntValue<'ctx> {
        self.untag(TAG_INTEGER, scope, value)
    }

    /// Untags a string value. The returrned PointerValue points to a value of `string_value_type`.
    pub(crate) fn untag_string(
        &self,
        scope: &Scope<'ctx>,
        value: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let untagged = self.untag(TAG_STRING, scope, value);
        self.builder
            .build_int_to_ptr(untagged, self.context.ptr_type(AddressSpace::default()), "")
            .unwrap()
    }

    pub(crate) fn branch_undefined(
        &self,
        value: PointerValue<'ctx>,
        uninit_block: BasicBlock<'ctx>,
        init_block: BasicBlock<'ctx>,
    ) {
        let tag = self.get_tag(value);
        let cmp = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.context.i8_type().const_int(TAG_UNDEFINED, false),
                "untag",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(cmp, uninit_block, init_block)
            .unwrap();
    }
}
