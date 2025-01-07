use crate::{scope::Scope, Codegen};
use inkwell::values::StructValue;
use trilogy_ir::ir;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_pattern_match(
        &self,
        _scope: &mut Scope<'ctx>,
        _pattern: &ir::Expression,
        _value: StructValue<'ctx>,
    ) {
        todo!()
    }
}
