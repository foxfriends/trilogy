use crate::{scope::Scope, Codegen};
use inkwell::values::PointerValue;
use trilogy_ir::ir::{self, Value};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_pattern_match(
        &self,
        scope: &mut Scope<'ctx>,
        pattern: &ir::Expression,
        value: PointerValue<'ctx>,
    ) {
        match &pattern.value {
            Value::Reference(id) => {
                let variable = self.variable(scope, id.id.clone());
                let value = self
                    .builder
                    .build_load(self.value_type(), value, id.id.name().unwrap_or(""))
                    .unwrap();
                self.builder.build_store(variable, value).unwrap();
            }
            Value::Conjunction(conj) => {
                self.compile_pattern_match(scope, &conj.0, value);
                self.compile_pattern_match(scope, &conj.1, value);
            }
            _ => todo!(),
        }
    }
}
