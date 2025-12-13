use inkwell::{
    AddressSpace,
    module::Linkage,
    values::{FunctionValue, InstructionValue, PointerValue},
};

use crate::codegen::Codegen;

macro_rules! core_binary_operator {
    ($name:ident) => {
        pub(crate) fn $name(
            &self,
            target: PointerValue<'ctx>,
            lhs: PointerValue<'ctx>,
            rhs: PointerValue<'ctx>,
        ) {
            let f = self.declare_core(stringify!($name), 2);
            self.call_core(target, f, &[lhs.into(), rhs.into()]);
        }
    };
}

macro_rules! core_unary_operator {
    ($name:ident) => {
        pub(crate) fn $name(&self, target: PointerValue<'ctx>, val: PointerValue<'ctx>) {
            let f = self.declare_core(stringify!($name), 1);
            self.call_core(target, f, &[val.into()]);
        }
    };
}

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

    /// Imported core procedures are the core.tri versions, so they must be called using regular
    /// procedure or function calling convention.
    fn import_core(&self, name: &str) -> FunctionValue<'ctx> {
        self.import_accessor(&format!("trilogy:core::{name}"))
    }

    pub(crate) fn reference_core(&self, name: &str) -> PointerValue<'ctx> {
        let target = self.allocate_value(name);
        let accessor = self.import_core(name);
        self.call_internal(target, accessor, &[]);
        target
    }

    core_binary_operator!(structural_eq);
    core_binary_operator!(structural_neq);
    core_binary_operator!(referential_eq);
    core_binary_operator!(referential_neq);
    core_binary_operator!(bitwise_or);
    core_binary_operator!(bitwise_and);
    core_binary_operator!(bitwise_xor);
    core_binary_operator!(shift_left);
    core_binary_operator!(shift_left_extend);
    core_binary_operator!(shift_left_contract);
    core_binary_operator!(shift_right);
    core_binary_operator!(shift_right_extend);
    core_binary_operator!(shift_right_contract);
    core_binary_operator!(glue);
    core_binary_operator!(lt);
    core_binary_operator!(gt);
    core_binary_operator!(lte);
    core_binary_operator!(gte);
    core_binary_operator!(add);
    core_binary_operator!(subtract);
    core_binary_operator!(multiply);
    core_binary_operator!(divide);
    core_binary_operator!(int_divide);
    core_binary_operator!(power);
    core_binary_operator!(rem);

    core_unary_operator!(boolean_not);
    core_unary_operator!(negate);
    core_unary_operator!(destruct);

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

    pub(crate) fn panic(&self, msg: PointerValue<'ctx>) -> InstructionValue<'ctx> {
        let f = self.declare_core("panic", 1);
        self.call_core(
            self.context.ptr_type(AddressSpace::default()).const_null(),
            f,
            &[msg.into()],
        )
    }

    pub(crate) fn to_string(&self, argument: PointerValue<'ctx>, name: &str) -> PointerValue<'ctx> {
        let to_string = self.reference_core("to_string");
        self.apply_function(to_string, argument, name)
    }

    pub(crate) fn compose(
        &self,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let compose = self.reference_core("compose");
        self.bind_temporary(rhs);
        let composed = self.apply_function(compose, lhs, "composing");
        let rhs = self.use_temporary_clone(rhs).unwrap();
        self.apply_function(composed, rhs, name)
    }

    pub(crate) fn member_access(
        &self,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let member_access = self.reference_core("member_access");
        self.bind_temporary(rhs);
        let composed = self.apply_function(member_access, lhs, "accessing");
        let rhs = self.use_temporary_clone(rhs).unwrap();
        self.apply_function(composed, rhs, name)
    }

    pub(crate) fn invert(&self, target: PointerValue<'ctx>, value: PointerValue<'ctx>) {
        let f = self.declare_core("bitwise_invert", 1);
        self.call_core(target, f, &[value.into()]);
    }

    pub(crate) fn elem(&self) -> PointerValue<'ctx> {
        self.reference_core("elem")
    }

    pub(crate) fn test_main(&self) -> PointerValue<'ctx> {
        self.reference_core("test_main")
    }
}
