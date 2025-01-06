use crate::{scope::Scope, Codegen};
use inkwell::values::StructValue;
use trilogy_ir::ir;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_pattern_match(
        &self,
        scope: &mut Scope<'ctx>,
        pattern: &ir::Expression,
        value: StructValue<'ctx>,
    ) {
        todo!()
    }
}
