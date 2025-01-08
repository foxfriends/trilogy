use crate::codegen::Codegen;
use inkwell::values::FunctionValue;

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn exit(&self) -> FunctionValue<'ctx> {
        if let Some(exit) = self.module.get_function("exit") {
            return exit;
        }

        self.module.add_function(
            "exit",
            self.context
                .void_type()
                .fn_type(&[self.context.i32_type().into()], false),
            None,
        )
    }
}
