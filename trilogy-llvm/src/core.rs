use inkwell::{
    AddressSpace,
    module::Linkage,
    values::{FunctionValue, InstructionValue, PointerValue},
};

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    /// These core functions let us call the C functions from core.c, which are backing the
    /// Trilogy core.tri procedures. This lets us take advantage of the simpler calling convention
    /// (no need for continuations), as the C functions definitely don't do anything like that.
    fn declare_core(&self, name: &str, arity: usize) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(name) {
            return func;
        }
        self.module
            .add_function(name, self.external_type(arity), Some(Linkage::External))
    }

    /// Imported core procedures are the core.tri versions, so they cost as much as a regular
    /// procedure call.
    fn import_core(&self, name: &str) -> FunctionValue<'ctx> {
        self.import_accessor(&format!("trilogy:core::{}", name))
    }

    pub(crate) fn structural_eq(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("structural_eq", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn structural_neq(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("structural_neq", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn referential_eq(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("referential_eq", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn referential_neq(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("referential_neq", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn not(&self, target: PointerValue<'ctx>, val: PointerValue<'ctx>) {
        let f = self.declare_core("boolean_not", 1);
        self.call_core(target, f, &[val.into()]);
    }

    #[expect(dead_code)]
    pub(crate) fn and(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("boolean_and", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    #[expect(dead_code)]
    pub(crate) fn or(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("boolean_or", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn bitwise_or(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("bitwise_or", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn bitwise_and(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("bitwise_and", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn bitwise_xor(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("bitwise_xor", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn shift_left(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("shift_left", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn shift_left_extend(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("shift_left_extend", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn shift_left_contract(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("shift_left_contract", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn shift_right(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("shift_right", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn shift_right_extend(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("shift_right_extend", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn shift_right_contract(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("shift_right_contract", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn member_access(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("member_access", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn member_assign(
        &self,
        target: PointerValue<'ctx>,
        container: PointerValue<'ctx>,
        key: PointerValue<'ctx>,
        value: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("member_assign", 3);
        self.call_core(target, f, &[container.into(), key.into(), value.into()]);
    }

    pub(crate) fn glue(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("glue", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn lt(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("lt", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn gt(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("gt", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn gte(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("gte", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn lte(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("lte", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn add(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("add", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn sub(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("subtract", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn mul(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("multiply", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn div(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("divide", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn int_div(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("int_divide", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn pow(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("power", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn rem(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("rem", 2);
        self.call_core(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn negate(&self, target: PointerValue<'ctx>, val: PointerValue<'ctx>) {
        let f = self.declare_core("negate", 1);
        self.call_core(target, f, &[val.into()]);
    }

    pub(crate) fn destruct(&self, target: PointerValue<'ctx>, value: PointerValue<'ctx>) {
        let f = self.declare_core("destruct", 1);
        self.call_core(target, f, &[value.into()]);
    }

    pub(crate) fn panic(&self, msg: PointerValue<'ctx>) -> InstructionValue<'ctx> {
        let f = self.declare_core("panic", 1);
        self.call_core(
            self.context.ptr_type(AddressSpace::default()).const_null(),
            f,
            &[msg.into()],
        )
    }

    pub(crate) fn to_string(&self, argument: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let target = self.allocate_value("to_string");
        let function = self.import_core("to_string");
        self.call_internal(target, function, &[]);
        self.call_procedure(target, &[argument], name)
    }

    pub(crate) fn invert(&self, target: PointerValue<'ctx>, value: PointerValue<'ctx>) {
        let f = self.declare_core("bitwise_invert", 1);
        self.call_core(target, f, &[value.into()]);
    }
}
