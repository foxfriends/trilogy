use crate::{scope::Scope, Codegen};
use inkwell::{
    values::{BasicMetadataValueEnum, PointerValue},
    AddressSpace,
};
use trilogy_ir::ir::{self, Builtin, Value};

impl<'ctx> Codegen<'ctx> {
    pub(crate) fn compile_expression(
        &self,
        scope: &mut Scope<'ctx>,
        expression: &ir::Expression,
    ) -> PointerValue<'ctx> {
        match &expression.value {
            Value::Unit => self.allocate_const(self.unit_const()),
            Value::Boolean(b) => self.allocate_const(self.bool_const(*b)),
            Value::Character(ch) => self.allocate_const(self.char_const(*ch)),
            Value::String(s) => self.allocate_const(self.string_const(s)),
            Value::Sequence(exprs) => {
                let mut value = self.allocate_const(self.unit_const());
                for expr in exprs {
                    value = self.compile_expression(scope, expr);
                }
                value
            }
            Value::Application(app) => self.compile_application(scope, app),
            Value::Builtin(val) => self.builtin_value(scope, *val),
            Value::Reference(val) => self.compile_reference(scope, val),
            _ => todo!(),
        }
    }

    pub(crate) fn builtin_value(
        &self,
        _scope: &mut Scope<'ctx>,
        _builtin: Builtin,
    ) -> PointerValue<'ctx> {
        todo!()
    }

    pub(crate) fn compile_application(
        &self,
        scope: &mut Scope<'ctx>,
        application: &ir::Application,
    ) -> PointerValue<'ctx> {
        if let Value::Builtin(builtin) = &application.function.value {
            return self.compile_apply_builtin(scope, *builtin, &application.argument);
        }

        let output = self
            .builder
            .build_alloca(self.value_type(), "retval")
            .unwrap();
        let function = self.compile_expression(scope, &application.function);
        let function = self.untag_function(scope, function);
        match &application.argument.value {
            // Procedure application
            Value::Pack(pack) => {
                let mut arguments = vec![output.into()];
                arguments.extend(pack.values.iter().map(|val| {
                    assert!(!val.is_spread);
                    BasicMetadataValueEnum::from(self.compile_expression(scope, &val.expression))
                }));
                self.builder
                    .build_indirect_call(self.function_type(), function, &arguments, "")
                    .unwrap();
            }
            // Function application
            _ => {
                let argument = self.compile_expression(scope, &application.argument);
                self.builder
                    .build_indirect_call(
                        self.function_type(),
                        function,
                        &[output.into(), argument.into()],
                        "",
                    )
                    .unwrap();
            }
        }
        output
    }

    pub(crate) fn compile_apply_builtin(
        &self,
        scope: &mut Scope<'ctx>,
        builtin: Builtin,
        expression: &ir::Expression,
    ) -> PointerValue<'ctx> {
        match builtin {
            Builtin::Return => {
                let argument = self.compile_expression(scope, expression);
                self.builder.build_store(scope.sret(), argument).unwrap();
                self.builder.build_return(None).unwrap();
                self.context.ptr_type(AddressSpace::default()).const_null()
            }
            Builtin::Typeof => {
                let argument = self.compile_expression(scope, expression);
                let tag = self
                    .builder
                    .build_struct_gep(self.value_type(), argument, 0, "tag")
                    .unwrap();
                let tag = self
                    .builder
                    .build_load(self.tag_type(), tag, "tag")
                    .unwrap()
                    .into_int_value();
                let tag_char = self
                    .builder
                    .build_int_add(self.context.i8_type().const_int(65, false), tag, "typeof")
                    .unwrap();
                self.char_value(tag_char)
            }
            _ => todo!(),
        }
    }

    pub(crate) fn compile_reference(
        &self,
        scope: &Scope<'ctx>,
        identifier: &ir::Identifier,
    ) -> PointerValue<'ctx> {
        if let Some(variable) = scope.variables.get(&identifier.id) {
            return *variable;
        } else if let Some(name) = identifier.id.name() {
            if let Some(function) = self.module.get_function(name) {
                let pointer = function.as_global_value().as_pointer_value();
                let stack = self.builder.build_alloca(self.value_type(), name).unwrap();
                self.builder
                    .build_store(stack, self.callable_value(pointer))
                    .unwrap();
                return stack;
            }
        }
        panic!("Unresolved variable");
    }
}
