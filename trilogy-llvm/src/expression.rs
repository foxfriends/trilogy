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
    ) -> Option<PointerValue<'ctx>> {
        self.set_span(expression.span);

        let output = match &expression.value {
            Value::Unit => self.allocate_const(self.unit_const()),
            Value::Boolean(b) => self.allocate_const(self.bool_const(*b)),
            Value::Atom(atom) => self.allocate_const(self.atom_const(atom.to_owned())),
            Value::Character(ch) => self.allocate_const(self.char_const(*ch)),
            Value::String(s) => self.allocate_const(self.string_const(s)),
            Value::Number(num) => {
                if num.value().im.is_zero() && num.value().re.is_integer() {
                    if let Some(int) = num.value().re.to_i64() {
                        self.allocate_const(self.int_const(int))
                    } else {
                        todo!("Support large integers")
                    }
                } else {
                    todo!("Support non-integers")
                }
            }
            Value::Bits(..) => todo!(),
            Value::Array(..) => todo!(),
            Value::Set(..) => todo!(),
            Value::Record(..) => todo!(),
            Value::ArrayComprehension(..) => todo!(),
            Value::SetComprehension(..) => todo!(),
            Value::RecordComprehension(..) => todo!(),
            Value::Sequence(exprs) => {
                self.di.push_block_scope(expression.span);
                let mut value = self.allocate_const(self.unit_const());
                for expr in exprs {
                    value = self.compile_expression(scope, expr)?;
                }
                self.di.pop_debug_scope();
                value
            }
            Value::Application(app) => self.compile_application(scope, app)?,
            Value::Builtin(val) => self.reference_builtin(scope, *val),
            Value::Reference(val) => self.compile_reference(scope, val),
            Value::ModuleAccess(access) => self.compile_module_access(scope, &access.0, &access.1),
            Value::IfElse(if_else) => self.compile_if_else(scope, if_else)?,
            Value::Assignment(assign) => self.compile_assignment(scope, assign)?,
            Value::While(..) => todo!(),
            Value::For(..) => todo!(),
            Value::Let(..) => todo!(),
            Value::Match(..) => todo!(),
            Value::Assert(..) => todo!(),
            Value::Fn(..) => todo!(),
            Value::Do(..) => todo!(),
            Value::Qy(..) => todo!(),
            Value::Handled(..) => todo!(),
            Value::End => todo!(),
            Value::Pack(..) => panic!("arbitrary packs are not permitted"),
            Value::Mapping(..) => panic!("arbitrary mappings are not permitted"),
            Value::Conjunction(..) => panic!("conjunction not permitted in expression context"),
            Value::Disjunction(..) => panic!("disjunction not permitted in expression context"),
            Value::Wildcard => panic!("wildcard not permitted in expression context"),
            Value::Query(..) => panic!("query not permitted in expression context"),
        };
        Some(output)
    }

    fn reference_builtin(&self, _scope: &mut Scope<'ctx>, _builtin: Builtin) -> PointerValue<'ctx> {
        todo!()
    }

    fn compile_application(
        &self,
        scope: &mut Scope<'ctx>,
        application: &ir::Application,
    ) -> Option<PointerValue<'ctx>> {
        match &application.function.value {
            Value::Builtin(builtin) if builtin.is_unary() => {
                return self.compile_apply_builtin(scope, *builtin, &application.argument)
            }
            Value::Application(app) => match &app.function.value {
                Value::Builtin(builtin) if builtin.is_binary() => {
                    return self.compile_apply_binary(
                        scope,
                        *builtin,
                        &app.argument,
                        &application.argument,
                    )
                }
                _ => {}
            },
            _ => {}
        };
        let function = self.compile_expression(scope, &application.function)?;
        match &application.argument.value {
            // Procedure application
            Value::Pack(pack) => {
                let arguments = pack
                    .values
                    .iter()
                    .map(|val| {
                        assert!(!val.is_spread);
                        Some(BasicMetadataValueEnum::from(
                            self.compile_expression(scope, &val.expression)?,
                        ))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(self.call_procedure(function, &arguments, ""))
            }
            // Function application
            _ => {
                let argument = self.compile_expression(scope, &application.argument)?;
                Some(self.apply_function(function, argument.into(), ""))
            }
        }
    }

    fn compile_module_access(
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
                return self.call_procedure(declared, &[], "");
            }
        }

        todo!()
    }

    fn compile_apply_builtin(
        &self,
        scope: &mut Scope<'ctx>,
        builtin: Builtin,
        expression: &ir::Expression,
    ) -> Option<PointerValue<'ctx>> {
        match builtin {
            Builtin::Return => {
                let argument = self.compile_expression(scope, expression)?;
                let val = self
                    .builder
                    .build_load(self.value_type(), argument, "retval")
                    .unwrap()
                    .into_struct_value();
                self.builder.build_store(scope.sret(), val).unwrap();
                self.builder
                    .build_unconditional_branch(scope.cleanup.unwrap())
                    .unwrap();
                None
            }
            Builtin::Exit => {
                let argument = self.compile_expression(scope, expression)?;
                _ = self.exit(argument);
                None
            }
            Builtin::Typeof => {
                let argument = self.compile_expression(scope, expression)?;
                let tag = self.get_tag(argument);
                let raw_atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                Some(self.raw_atom_value(raw_atom))
            }
            _ => todo!(),
        }
    }

    fn compile_apply_binary(
        &self,
        scope: &mut Scope<'ctx>,
        builtin: Builtin,
        lhs: &ir::Expression,
        rhs: &ir::Expression,
    ) -> Option<PointerValue<'ctx>> {
        match builtin {
            Builtin::StructuralEquality => {
                let lhs = self.compile_expression(scope, lhs);
                let rhs = self.compile_expression(scope, rhs);
                Some(self.structural_eq(lhs?, rhs?, "eq"))
            }
            Builtin::ReferenceEquality => {
                let lhs = self.compile_expression(scope, lhs);
                let rhs = self.compile_expression(scope, rhs);
                Some(self.referential_eq(lhs?, rhs?, "eq"))
            }
            _ => todo!(),
        }
    }

    fn compile_assignment(
        &self,
        scope: &mut Scope<'ctx>,
        assign: &ir::Assignment,
    ) -> Option<PointerValue<'ctx>> {
        match &assign.lhs.value {
            Value::Reference(variable) => {
                let value = self.compile_expression(scope, &assign.rhs)?;
                let pointer = *scope.variables.get(&variable.id).unwrap();
                self.builder.build_store(pointer, value).unwrap();
                Some(pointer)
            }
            Value::Application(..) => todo!(),
            _ => panic!("invalid lvalue in assignment"),
        }
    }

    fn compile_reference(
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
                Head::Constant | Head::Procedure(..) => {
                    let global_name =
                        format!("{}::{name}", self.module.get_name().to_str().unwrap());
                    let function = self.module.get_function(&global_name).unwrap();
                    self.call_procedure(function, &[], name)
                }
                _ => todo!(),
            }
        }
    }

    fn compile_if_else(
        &self,
        scope: &mut Scope<'ctx>,
        if_else: &ir::IfElse,
    ) -> Option<PointerValue<'ctx>> {
        let condition = self.compile_expression(scope, &if_else.condition)?;
        let if_true = self.context.append_basic_block(scope.function, "if_true");
        let if_false = self.context.append_basic_block(scope.function, "if_false");
        let if_cont = self.context.append_basic_block(scope.function, "if_cont");
        let condition = self.trilogy_boolean_untag(condition, "");
        self.builder
            .build_conditional_branch(condition, if_true, if_false)
            .unwrap();

        self.builder.position_at_end(if_true);
        let when_true = self.compile_expression(scope, &if_else.when_true);
        let if_true = self.builder.get_insert_block().unwrap();
        if !if_true.get_last_instruction().unwrap().is_terminator() {
            self.builder.build_unconditional_branch(if_cont).unwrap();
        }

        self.builder.position_at_end(if_false);
        let when_false = self.compile_expression(scope, &if_else.when_false);
        let if_false = self.builder.get_insert_block().unwrap();
        if !if_false.get_last_instruction().unwrap().is_terminator() {
            self.builder.build_unconditional_branch(if_cont).unwrap();
        }

        self.builder.position_at_end(if_cont);
        match (when_true, when_false) {
            (None, None) => None,
            (Some(when_true), Some(when_false)) => {
                let phi = self
                    .builder
                    .build_phi(self.context.ptr_type(AddressSpace::default()), "if_eval")
                    .unwrap();
                phi.add_incoming(&[(&when_true, if_true), (&when_false, if_false)]);
                Some(phi.as_basic_value().into_pointer_value())
            }
            (Some(v), None) | (None, Some(v)) => Some(v),
        }
    }
}
