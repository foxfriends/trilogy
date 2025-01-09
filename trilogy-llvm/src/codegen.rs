#![expect(dead_code, reason = "WIP")]

use crate::scope::Scope;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{FunctionType, IntType, StructType},
    values::{
        BasicMetadataValueEnum, BasicValue, FunctionValue, IntValue, PointerValue, StructValue,
    },
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
const TAG_CALLABLE: u64 = 12;

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
    pub(crate) fn new(context: &'ctx Context) -> Self {
        let codegen = Codegen {
            module: context.create_module("trilogy:runtime"),
            builder: context.create_builder(),
            context,
        };

        let submodule = codegen.sub("trilogy:c");
        submodule.std_libc();

        codegen.module.link_in_module(submodule.module).unwrap();
        codegen
    }

    pub(crate) fn compile_entrypoint(&self, entrymodule: &str, entrypoint: &str) {
        let main_wrapper =
            self.module
                .add_function("main", self.context.i8_type().fn_type(&[], false), None);
        let basic_block = self.context.append_basic_block(main_wrapper, "entry");
        self.builder.position_at_end(basic_block);
        let main = self
            .module
            .get_function(&format!("{entrymodule}::{entrypoint}"))
            .unwrap();
        let output = self
            .builder
            .build_alloca(self.value_type(), "output")
            .unwrap();
        self.builder
            .build_direct_call(main, &[output.into()], "main")
            .unwrap();
        let exitcode = self
            .builder
            .build_struct_gep(self.value_type(), output, 0, "exitcode")
            .unwrap();
        let exitcode = self
            .builder
            .build_load(self.tag_type(), exitcode, "exitcode")
            .unwrap();
        self.builder.build_return(Some(&exitcode)).unwrap();
    }

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
        self.context
            .struct_type(&[self.tag_type().into(), self.payload_type().into()], false)
    }

    pub(crate) fn tag_type(&self) -> IntType<'ctx> {
        self.context.i8_type()
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
                self.context.i64_type().into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        )
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

    pub(crate) fn atom_const(&self, id: u64) -> StructValue<'ctx> {
        self.value_type().const_named_struct(&[
            self.tag_type().const_int(TAG_ATOM, false).into(),
            self.payload_type().const_int(id, false).into(),
        ])
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
        let body = self
            .builder
            .build_struct_gep(self.value_type(), pointer, 1, "")
            .unwrap();
        let int = self
            .builder
            .build_int_z_extend(value, self.payload_type(), "")
            .unwrap();
        self.builder.build_store(body, int).unwrap();
        pointer
    }

    pub(crate) fn string_const(&self, value: &str) -> StructValue<'ctx> {
        // SAFETY: it seems the only restriction is that this must not be called outside of a
        // function, which is not checked but we will never do it.
        let string = unsafe { self.builder.build_global_string(value, "").unwrap() };
        // TODO: this is wrong... needs to probably construct fully, not using const
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

    pub(crate) fn add_procedure(
        &self,
        name: &str,
        arity: usize,
        exported: bool,
    ) -> FunctionValue<'ctx> {
        let procedure = self.module.add_function(
            name,
            self.procedure_type(arity),
            if exported {
                Some(Linkage::External)
            } else {
                Some(Linkage::Private)
            },
        );
        procedure.add_attribute(
            AttributeLoc::Param(0),
            self.context.create_type_attribute(
                Attribute::get_named_enum_kind_id("sret"),
                self.value_type().into(),
            ),
        );
        procedure.get_nth_param(0).unwrap().set_name("sretptr");
        procedure
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

    pub(crate) fn function_type(&self) -> FunctionType<'ctx> {
        self.procedure_type(1)
    }

    pub(crate) fn variable(&self, scope: &mut Scope<'ctx>, id: Id) -> PointerValue<'ctx> {
        if scope.variables.contains_key(&id) {
            return *scope.variables.get(&id).unwrap();
        }

        let builder = self.context.create_builder();
        let entry = scope.function.get_first_basic_block().unwrap();
        match entry.get_first_instruction() {
            Some(instruction) => builder.position_before(&instruction),
            None => builder.position_at_end(entry),
        }
        let variable = builder
            .build_alloca(self.value_type(), &id.to_string())
            .unwrap();
        scope.variables.insert(id, variable);
        variable
    }

    pub(crate) fn untag_function(
        &self,
        scope: &Scope<'ctx>,
        value: PointerValue<'ctx>,
    ) -> PointerValue<'ctx> {
        let then_block = self.context.append_basic_block(scope.function, "then");
        let else_block = self.context.append_basic_block(scope.function, "else");
        let cont_block = self.context.append_basic_block(scope.function, "untagged");
        let tag = self.get_tag(value);
        let cmp = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.context.i8_type().const_int(TAG_CALLABLE, false),
                "untag",
            )
            .unwrap();
        self.builder
            .build_conditional_branch(cmp, then_block, else_block)
            .unwrap();

        self.builder.position_at_end(then_block);
        let then_val = self.get_payload(value);
        let then_val = self
            .builder
            .build_bit_cast(then_val, self.context.i64_type(), "")
            .unwrap()
            .into_int_value();
        let then_val = self
            .builder
            .build_int_to_ptr(then_val, self.context.ptr_type(AddressSpace::default()), "")
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

        self.builder.position_at_end(else_block);
        // TODO: handle mismatches... we're just ignoring them for now
        // which leads to some pretty NASTY UB
        let else_val = self.get_payload(value);
        let else_val = self
            .builder
            .build_bit_cast(else_val, self.context.i64_type(), "")
            .unwrap()
            .into_int_value();
        let else_val = self
            .builder
            .build_int_to_ptr(else_val, self.context.ptr_type(AddressSpace::default()), "")
            .unwrap();
        self.builder.build_unconditional_branch(cont_block).unwrap();

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

    pub(crate) fn compile_module(&self, file: &str, module: &ir::Module) -> Codegen<'ctx> {
        let subcontext = self.sub(file);
        for definition in module.definitions() {
            match &definition.item {
                DefinitionItem::Procedure(procedure) => {
                    subcontext.compile_procedure(file, procedure, definition.is_exported);
                }
                _ => todo!(),
            }
        }
        subcontext
    }
}
