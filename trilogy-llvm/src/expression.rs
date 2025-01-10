use crate::{scope::Scope, Codegen};
use inkwell::{
    values::{BasicMetadataValueEnum, PointerValue},
    AddressSpace,
};
use trilogy_ir::ir::{self, Builtin, Value};
use trilogy_parser::syntax;

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
            Value::ModuleAccess(access) => self.compile_module_access(scope, &access.0, &access.1),
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

        let function = self.compile_expression(scope, &application.function);
        let function = self.untag_function(scope, function);
        match &application.argument.value {
            // Procedure application
            Value::Pack(pack) => {
                let arguments: Vec<_> = pack
                    .values
                    .iter()
                    .map(|val| {
                        assert!(!val.is_spread);
                        BasicMetadataValueEnum::from(
                            self.compile_expression(scope, &val.expression),
                        )
                    })
                    .collect();
                self.call_procedure(function, &arguments, "")
            }
            // Function application
            _ => {
                let argument = self.compile_expression(scope, &application.argument);
                self.apply_function(function, argument.into(), "")
            }
        }
    }

    pub(crate) fn compile_module_access(
        &self,
        _scope: &mut Scope<'ctx>,
        module_ref: &ir::Expression,
        ident: &syntax::Identifier,
    ) -> PointerValue<'ctx> {
        // Possibly a static module reference, which we can support very easily and efficiently
        if let Value::Reference(name) = &module_ref.value {
            if let Some(name) = self.external_modules.get(&name.id) {
                let declared = self
                    .module
                    .get_function(&format!("{}::{}", name, ident.as_ref()))
                    .unwrap();
                return self.callable_value(declared.as_global_value().as_pointer_value());
            }
        }

        todo!()
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
                let val = self
                    .builder
                    .build_load(self.value_type(), argument, "retval")
                    .unwrap()
                    .into_struct_value();
                self.builder.build_store(scope.sret(), val).unwrap();
                self.builder.build_return(None).unwrap();
                self.context.ptr_type(AddressSpace::default()).const_null()
            }
            Builtin::Exit => {
                let exit = self.exit();
                let argument = self.compile_expression(scope, expression);
                let output = self.call_procedure(exit, &[argument.into()], "exit");
                self.builder.build_unreachable().unwrap();
                output
            }
            Builtin::Typeof => {
                let argument = self.compile_expression(scope, expression);
                let tag = self.get_tag(argument);
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
            let global_name = format!("{}::{name}", self.module.get_name().to_str().unwrap());
            if let Some(function) = self.module.get_function(&global_name) {
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
