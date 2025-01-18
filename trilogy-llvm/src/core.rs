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
        self.module
            .add_function(&name, self.procedure_type(arity), Some(Linkage::External))
    }

    pub(crate) fn structural_eq(
        &self,
        lhs: PointerValue<'ctx>,
        rhs: PointerValue<'ctx>,
        name: &str,
    ) -> PointerValue<'ctx> {
        let f = self.declare_core("structural_eq", 2);
        self.call_procedure(f, &[lhs.into(), rhs.into()], name)
    }
}
