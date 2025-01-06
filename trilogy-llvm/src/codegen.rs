use std::ops::Add;

use crate::scope::Scope;
use inkwell::{
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{FunctionType, StructType, VectorType},
    values::{BasicValueEnum, PointerValue, StructValue},
    AddressSpace, IntPredicate,
};
use trilogy_ir::{
    ir::{self, DefinitionItem},
    Id,
};

pub(crate) struct Codegen<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
}

const TAG_UNIT: u64 = 0;
const TAG_BOOL: u64 = 1;
const TAG_ATOM: u64 = 2;
const TAG_CHAR: u64 = 3;
const TAG_STRING: u64 = 4;
const TAG_NUMBER: u64 = 5;
const TAG_BITS: u64 = 6;
const TAG_STRUCT: u64 = 7;
const TAG_TUPLE: u64 = 8;
const TAG_ARRAY: u64 = 9;
const TAG_SET: u64 = 10;
const TAG_RECORD: u64 = 11;
const TAG_FUNCTION: u64 = 12;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn new(context: &'ctx Context) -> Self {
        let codegen = Codegen {
            module: context.create_module("trilogy:runtime"),
            builder: context.create_builder(),
            context,
        };

        codegen
    }

    /// Every value in a Trilogy program is represented as an instance of this "union" struct.
    /// The first field is the tag, and the second is the value, which is often a pointer
    /// to some underlying value of which the format is known to the runtime but not the
    /// Trilogy end user.
    ///
    /// The interpretation of this value is as follows:
    /// * Tag `0` = `unit`; the value field is empty.
    /// * Tag `1` = `bool`; the value field is `0x00000001` for true and `0x00000000` for false.
    /// * Tag `2` = `atom`; the value field is opaque, and is used as a unique index for this atom. This puts an implicit limit of `u64::MAX_VALUE` possible atoms in a program (which should be more than enough).
    /// * Tag `3` = `char`; the value field is `0x0000abcd` where `0xabcd` is the Unicode code point of the character.
    /// * Tag `4` = `string`; the value field is a pointer to a struct of `{ i64 length, [i8 x length] bytes }`, which is the string encoded in UTF-8 format.
    /// * Tag `5` = `number`; the value field is a pointer to an arbitrary precision number value.
    /// * Tag `6` = `bits`; the value field is a pointer to a struct of `{ i64 length, [i1 x length] bits }` which are the literal bits.
    /// * Tag `7` = `struct`; the value field is a pointer to a struct of `{ i64 tag, ptr value }` which is the atom ID, followed by a pointer to another value.
    /// * Tag `8` = `tuple`; the value field is an array of two pointers to the two values in this tuple.
    /// * Tag `9` = `array`; the value field is a pointer to a struct of `{ i64 length, [ptr x length] items }`.
    /// * Tag `10` = `set`; the value field is a pointer to a struct of `{ i64 length, [ptr x length] items }`.
    /// * Tag `11` = `record`; the value field is a pointer to a struct of `{ i64 length, [[ptr x 2] x length] items }`.
    /// * Tag `12` = `callable`; the value field is a pointer to a function.
    pub(crate) fn value_type(&self) -> StructType<'ctx> {
        self.context.struct_type(
            &[
                self.context.i8_type().into(),
                self.context.i8_type().vec_type(8).into(),
            ],
            false,
        )
    }

    pub(crate) fn string_value_type(&self) -> StructType<'ctx> {
        self.context.struct_type(
            &[
                self.context.i64_type().into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        )
    }

    pub(crate) fn unit_value(&self) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.context.i8_type().const_int(TAG_UNIT, false).into(),
            VectorType::const_vector(&[
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
            ])
            .into(),
        ])
    }

    pub(crate) fn bool_value(&self, value: bool) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.context.i8_type().const_int(TAG_BOOL, false).into(),
            VectorType::const_vector(&[
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context
                    .i8_type()
                    .const_int(if value { 1 } else { 0 }, false),
            ])
            .into(),
        ])
    }

    pub(crate) fn atom_value(&self, id: u64) -> StructValue<'ctx> {
        let bytes = id.to_be_bytes();
        self.value_type().const_named_struct(&[
            self.context.i8_type().const_int(TAG_ATOM, false).into(),
            VectorType::const_vector(&[
                self.context.i8_type().const_int(bytes[0] as u64, false),
                self.context.i8_type().const_int(bytes[1] as u64, false),
                self.context.i8_type().const_int(bytes[2] as u64, false),
                self.context.i8_type().const_int(bytes[3] as u64, false),
                self.context.i8_type().const_int(bytes[4] as u64, false),
                self.context.i8_type().const_int(bytes[5] as u64, false),
                self.context.i8_type().const_int(bytes[6] as u64, false),
                self.context.i8_type().const_int(bytes[7] as u64, false),
            ])
            .into(),
        ])
    }

    pub(crate) fn char_value(&self, value: char) -> StructValue<'ctx> {
        let value = value as u32;
        let bytes = value.to_be_bytes();
        self.value_type().const_named_struct(&[
            self.context.i8_type().const_int(TAG_CHAR, false).into(),
            VectorType::const_vector(&[
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(0, false),
                self.context.i8_type().const_int(bytes[0] as u64, false),
                self.context.i8_type().const_int(bytes[1] as u64, false),
                self.context.i8_type().const_int(bytes[2] as u64, false),
                self.context.i8_type().const_int(bytes[3] as u64, false),
            ])
            .into(),
        ])
    }

    pub(crate) fn string_value(&self, value: &str) -> StructValue<'ctx> {
        // SAFETY: it seems the only restriction is that this must not be called outside of a
        // function, which is not checked but we will never do it.
        let string = unsafe { self.builder.build_global_string(value, "").unwrap() };
        let string = self.string_value_type().const_named_struct(&[
            self.context
                .i64_type()
                .const_int(value.as_bytes().len() as u64, false)
                .into(),
            string.as_pointer_value().into(),
        ]);
        let global = self.module.add_global(self.string_value_type(), None, "");
        global.set_initializer(&string);
        let int = self
            .builder
            .build_ptr_to_int(global.as_pointer_value(), self.context.i64_type(), "")
            .unwrap();
        let vec = self
            .builder
            .build_bit_cast(int, self.context.i8_type().vec_type(8), "")
            .unwrap();
        self.value_type().const_named_struct(&[
            self.context.i8_type().const_int(TAG_STRING, false).into(),
            vec.into(),
        ])
    }

    pub(crate) fn procedure_type(&self, arity: usize) -> FunctionType<'ctx> {
        self.value_type()
            .fn_type(&vec![self.value_type().into(); arity], false)
    }

    pub(crate) fn function_type(&self) -> FunctionType<'ctx> {
        self.procedure_type(1)
    }

    pub(crate) fn variable(&self, scope: Scope<'ctx>, id: Id) -> PointerValue<'ctx> {
        if scope.variables.contains_key(&id) {
            return *scope.variables.get(&id).unwrap();
        }

        let builder = self.context.create_builder();
        let entry = scope.function.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(instruction) => builder.position_before(&instruction),
            None => builder.position_at_end(entry),
        }
        builder
            .build_alloca(self.value_type(), &id.to_string())
            .unwrap()
    }

    pub(crate) fn untag_function(
        &self,
        scope: &Scope<'ctx>,
        value: StructValue,
    ) -> PointerValue<'ctx> {
        assert_eq!(value.get_type(), self.value_type());
        let then_block = self.context.append_basic_block(scope.function, "then");
        let else_block = self.context.append_basic_block(scope.function, "else");
        let cont_block = self.context.append_basic_block(scope.function, "untagged");

        let tag_field = value.get_field_at_index(0).unwrap().into_int_value();
        let cmp = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag_field,
                self.context.i8_type().const_int(TAG_FUNCTION, false),
                "untag",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(cmp, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        let then_val = value.get_field_at_index(1).unwrap().into_vector_value();
        let then_val = self
            .builder
            .build_bit_cast(then_val, self.context.i64_type(), "")
            .unwrap()
            .into_int_value();
        let then_val = self
            .builder
            .build_int_to_ptr(then_val, self.context.ptr_type(AddressSpace::default()), "")
            .unwrap();

        self.builder.position_at_end(else_block);
        // TODO: handle mismatches... we're just ignoring them for now
        // which leads to some pretty NASTY UB
        let else_val = value.get_field_at_index(1).unwrap().into_vector_value();
        let else_val = self
            .builder
            .build_bit_cast(else_val, self.context.i64_type(), "")
            .unwrap()
            .into_int_value();
        let else_val = self
            .builder
            .build_int_to_ptr(else_val, self.context.ptr_type(AddressSpace::default()), "")
            .unwrap();

        self.builder.position_at_end(cont_block);
        let phi = self
            .builder
            .build_phi(self.context.ptr_type(AddressSpace::default()), "untagtmp")
            .unwrap();
        phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);
        phi.as_basic_value().into_pointer_value()
    }

    fn sub(&self, name: &str) -> Codegen<'ctx> {
        Codegen {
            context: self.context,
            module: self.context.create_module(name),
            builder: self.context.create_builder(),
        }
    }

    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) {
        let subcontext = self.sub(&format!("file:{}", file));
        for definition in module.definitions() {
            let linkage = if definition.is_exported {
                Linkage::External
            } else {
                Linkage::Private
            };
            match &definition.item {
                DefinitionItem::Procedure(procedure) => {
                    subcontext.compile_procedure(procedure, linkage);
                }
                _ => todo!(),
            }
        }
        self.module.link_in_module(subcontext.module).unwrap();
    }
}
