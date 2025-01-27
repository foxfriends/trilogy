use inkwell::{
    module::Linkage,
    values::{FunctionValue, PointerValue},
};

use crate::codegen::Codegen;

impl<'ctx> Codegen<'ctx> {
    fn declare_core(&self, name: &str, arity: usize) -> FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function(name) {
            return func;
        }
        self.module.add_function(
            name,
            self.procedure_type(arity, false),
            Some(Linkage::External),
        )
    }

    pub(crate) fn structural_eq(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("structural_eq", 2);
        self.call_internal(target, f, &[lhs.into(), rhs.into()]);
    }

    pub(crate) fn referential_eq(
        &self,
        target: PointerValue<'ctx>,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
    ) {
        let f = self.declare_core("referential_eq", 2);
        self.call_internal(target, f, &[lhs.into(), rhs.into()]);
    }
}
