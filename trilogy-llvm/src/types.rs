use crate::codegen::Codegen;
use bitvec::field::BitField;
use inkwell::{
    basic_block::BasicBlock,
    types::{FunctionType, IntType, StructType},
    values::{
        BasicMetadataValueEnum, BasicValue, FunctionValue, IntValue, PointerValue, StructValue,
    },
    AddressSpace, IntPredicate,
};
use trilogy_ir::ir::Bits;

pub(crate) const TAG_UNDEFINED: u64 = 0;
pub(crate) const TAG_UNIT: u64 = 1;
pub(crate) const TAG_BOOL: u64 = 2;
pub(crate) const TAG_ATOM: u64 = 3;
pub(crate) const TAG_CHAR: u64 = 4;
pub(crate) const TAG_STRING: u64 = 5;
pub(crate) const TAG_NUMBER: u64 = 6;
pub(crate) const TAG_BITS: u64 = 7;
pub(crate) const TAG_STRUCT: u64 = 8;
pub(crate) const TAG_TUPLE: u64 = 9;
pub(crate) const TAG_ARRAY: u64 = 10;
pub(crate) const TAG_SET: u64 = 11;
pub(crate) const TAG_RECORD: u64 = 12;
pub(crate) const TAG_CALLABLE: u64 = 13;

pub(crate) trait TrilogyCallable<'ctx> {
    fn build_procedure_call(self, codegen: &Codegen<'ctx>, args: &[BasicMetadataValueEnum<'ctx>]);

    fn build_function_call(self, codegen: &Codegen<'ctx>, args: &[BasicMetadataValueEnum<'ctx>]);
}

impl<'ctx> TrilogyCallable<'ctx> for PointerValue<'ctx> {
    fn build_procedure_call(self, codegen: &Codegen<'ctx>, args: &[BasicMetadataValueEnum<'ctx>]) {
        let callable = codegen.trilogy_callable_untag(self, "");
        let function = codegen.trilogy_procedure_untag(callable, args.len() - 1, "");
        codegen
            .builder
            .build_indirect_call(codegen.procedure_type(args.len() - 1), function, args, "")
            .unwrap();
    }

    fn build_function_call(self, codegen: &Codegen<'ctx>, args: &[BasicMetadataValueEnum<'ctx>]) {
        let callable = codegen.trilogy_callable_untag(self, "");
        let function = codegen.trilogy_function_untag(callable, "");
        codegen
            .builder
            .build_indirect_call(codegen.procedure_type(1), function, args, "")
            .unwrap();
    }
}

impl<'ctx> TrilogyCallable<'ctx> for FunctionValue<'ctx> {
    fn build_procedure_call(self, codegen: &Codegen<'ctx>, args: &[BasicMetadataValueEnum<'ctx>]) {
        codegen.builder.build_call(self, args, "").unwrap();
    }

    fn build_function_call(self, codegen: &Codegen<'ctx>, args: &[BasicMetadataValueEnum<'ctx>]) {
        codegen.builder.build_call(self, args, "").unwrap();
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

    pub(crate) fn payload_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
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

    pub(crate) fn bits_value_type(&self) -> StructType<'ctx> {
        self.context
            .struct_type(&[self.usize_type().into(), self.usize_type().into()], false)
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

    pub(crate) fn char_const(&self, value: char) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_CHAR, false).into(),
            self.payload_type().const_int(value as u64, false).into(),
        ])
    }

    pub(crate) fn int_const(&self, value: i64) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_NUMBER, false).into(),
            self.payload_type().const_int(value as u64, false).into(),
        ])
    }

    pub(crate) fn string_const(&self, value: &str) -> StructValue<'ctx> {
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
                .const_int(value.len() as u64, false)
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

    pub(crate) fn bits_const(&self, value: &Bits) -> StructValue<'ctx> {
        let bit_len = value.value().len();
        let bytes: Vec<u8> = value
            .value()
            .chunks(8)
            .map(|s| s.load_be::<u8>() << (8 - s.len()))
            .collect();
        let byte_len = bytes.len();
        let bitstring =
            self.module
                .add_global(self.context.i8_type().array_type(byte_len as u32), None, "");
        bitstring.set_initializer(&self.context.const_string(&bytes, false));
        bitstring.set_constant(true);
        let bitstring = self.bits_value_type().const_named_struct(&[
            self.context
                .i64_type()
                .const_int(bit_len as u64, false)
                .into(),
            bitstring.as_pointer_value().into(),
        ]);
        let global = self.module.add_global(self.bits_value_type(), None, "");
        global.set_initializer(&bitstring);
        global.set_constant(true);
        let int = self
            .builder
            .build_ptr_to_int(global.as_pointer_value(), self.payload_type(), "")
            .unwrap();
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_BITS, false).into(),
            int.into(),
        ])
    }

    pub(crate) fn procedure_type(&self, arity: usize) -> FunctionType<'ctx> {
        self.context.void_type().fn_type(
            &vec![self.context.ptr_type(AddressSpace::default()).into(); arity + 1],
            false,
        )
    }

    #[expect(dead_code)]
    pub(crate) fn function_type(&self) -> FunctionType<'ctx> {
        self.procedure_type(1)
    }

    pub(crate) fn call_procedure(
        &self,
        target: PointerValue<'ctx>,
        procedure: impl TrilogyCallable<'ctx>,
        arguments: &[BasicMetadataValueEnum<'ctx>],
    ) {
        let mut args = vec![target.into()];
        args.extend_from_slice(arguments);
        procedure.build_procedure_call(self, &args);
    }

    pub(crate) fn apply_function(
        &self,
        target: PointerValue<'ctx>,
        function: impl TrilogyCallable<'ctx>,
        argument: BasicMetadataValueEnum<'ctx>,
    ) {
        function.build_function_call(self, &[target.into(), argument]);
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
