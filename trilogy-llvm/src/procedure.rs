use crate::Codegen;
use inkwell::module::Linkage;
use trilogy_ir::ir;

impl Codegen<'_> {
    pub(crate) fn compile_procedure(&self, procedure: &ir::ProcedureDefinition, linkage: Linkage) {
        let fn_type = self.value_type().fn_type(
            &vec![self.value_type().into(); procedure.overloads[0].parameters.len()],
            false,
        );
        let function =
            self.module
                .add_function(&procedure.name.to_string(), fn_type, Some(linkage));
    }
}
