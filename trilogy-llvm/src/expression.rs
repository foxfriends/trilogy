use crate::{scope::Scope, Codegen};
use inkwell::values::StructValue;
use trilogy_ir::ir::{self, Builtin, Value};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_expression(
        &self,
        scope: &mut Scope<'ctx>,
        expression: &ir::Expression,
    ) -> StructValue<'ctx> {
        match &expression.value {
            Value::Unit => self.unit_value(),
            Value::Boolean(b) => self.bool_value(*b),
            Value::Character(ch) => self.char_value(*ch),
            Value::String(s) => self.string_value(s),
            Value::Sequence(exprs) => {
                let mut value = self.unit_value();
                for expr in exprs {
                    value = self.compile_expression(scope, expr);
                }
                value
            }
            Value::Application(app) => self.compile_application(scope, &*app),
            Value::Builtin(val) => self.builtin_value(scope, *val),
            _ => todo!(),
        }
    }

    pub(crate) fn builtin_value(
        &self,
        _scope: &mut Scope<'ctx>,
        builtin: Builtin,
    ) -> StructValue<'ctx> {
        match builtin {
            _ => todo!(),
        }
    }

    pub(crate) fn compile_application(
        &self,
        scope: &mut Scope<'ctx>,
        application: &ir::Application,
    ) -> StructValue<'ctx> {
        if let Value::Builtin(builtin) = &application.function.value {
            return self.compile_apply_builtin(scope, *builtin, &application.argument);
        }

        let function = self.compile_expression(scope, &application.function);
        let argument = self.compile_expression(scope, &application.argument);
        let function = self.untag_function(scope, function);
        self.builder
            .build_indirect_call(self.function_type(), function, &[argument.into()], "")
            .unwrap()
            .try_as_basic_value()
            .unwrap_left()
            .into_struct_value()
    }

    pub(crate) fn compile_apply_builtin(
        &self,
        scope: &mut Scope<'ctx>,
        builtin: Builtin,
        expression: &ir::Expression,
    ) -> StructValue<'ctx> {
        match builtin {
            Builtin::Return => {
                let argument = self.compile_expression(scope, expression);
                self.builder.build_return(Some(&argument)).unwrap();
                self.unit_value()
            }
            _ => todo!(),
        }
    }
}
