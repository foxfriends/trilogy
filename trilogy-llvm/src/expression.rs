use crate::{codegen::Head, scope::Scope, Codegen};
use inkwell::{
    values::{BasicMetadataValueEnum, PointerValue},
    AddressSpace,
};
use num::{ToPrimitive, Zero};
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
            Value::Number(num) if num.value().im.is_zero() && num.value().re.is_integer() => {
                if let Some(int) = num.value().re.to_i64() {
                    self.allocate_const(self.int_const(int))
                } else {
                    todo!("Support non-integers and large integers")
                }
            }
            Value::Atom(atom) => self.allocate_const(self.atom_const(atom.to_owned())),
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
        match &application.function.value {
            Value::Builtin(builtin) => {
                self.compile_apply_builtin(scope, *builtin, &application.argument)
            }
            _ => {
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
            if let Some(Head::Module(name)) = self.globals.get(&name.id) {
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
                let raw_atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                self.raw_atom_value(raw_atom)
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
            *variable
        } else {
            let name = identifier.id.name().unwrap();
            match self
                .globals
                .get(&identifier.id)
                .expect("Unresolved variable")
            {
                Head::Constant => {
                    let global_name =
                        format!("{}::{name}", self.module.get_name().to_str().unwrap());
                    let function = self.module.get_function(&global_name).unwrap();
                    let stack = self.builder.build_alloca(self.value_type(), name).unwrap();
                    self.builder
                        .build_call(function, &[stack.into()], "")
                        .unwrap();
                    stack
                }
                Head::Procedure(..) => {
                    let global_name =
                        format!("{}::{name}", self.module.get_name().to_str().unwrap());
                    let function = self.module.get_function(&global_name).unwrap();
                    let pointer = function.as_global_value().as_pointer_value();
                    let stack = self.builder.build_alloca(self.value_type(), name).unwrap();
                    self.builder
                        .build_store(stack, self.callable_value(pointer))
                        .unwrap();
                    stack
                }
                _ => todo!(),
            }
        }
    }
}
