use crate::{codegen::Head, scope::Scope, Codegen};
use inkwell::values::PointerValue;
use num::{ToPrimitive, Zero};
use trilogy_ir::ir::{self, Builtin, QueryValue, Value};
use trilogy_parser::syntax;

impl<'ctx> Codegen<'ctx> {
    #[must_use = "allocated value must be destroyed"]
    pub(crate) fn allocate_expression(
        &self,
        scope: &mut Scope<'ctx>,
        expression: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        let target = self.allocate_value(name);
        self.compile_expression(scope, target, expression)?;
        Some(target)
    }

    #[must_use = "must acknowldge continuation of control flow"]
    pub(crate) fn compile_expression(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        expression: &ir::Expression,
    ) -> Option<()> {
        let prev = self.set_span(expression.span);

        match &expression.value {
            Value::Unit => {
                self.builder.build_store(target, self.unit_const()).unwrap();
            }
            Value::Boolean(b) => {
                self.builder
                    .build_store(target, self.bool_const(*b))
                    .unwrap();
            }
            Value::Atom(atom) => {
                self.builder
                    .build_store(target, self.atom_const(atom.to_owned()))
                    .unwrap();
            }
            Value::Character(ch) => {
                self.builder
                    .build_store(target, self.char_const(*ch))
                    .unwrap();
            }
            Value::String(s) => {
                self.string_const(target, s);
            }
            Value::Number(num) => {
                if num.value().im.is_zero() && num.value().re.is_integer() {
                    if let Some(int) = num.value().re.to_i64() {
                        self.builder
                            .build_store(target, self.int_const(int))
                            .unwrap();
                    } else {
                        todo!("Support large integers")
                    }
                } else {
                    todo!("Support non-integers")
                }
            }
            Value::Bits(b) => {
                self.builder
                    .build_store(target, self.bits_const(b))
                    .unwrap();
            }
            Value::Array(arr) => self.compile_array(scope, target, arr)?,
            Value::Set(..) => todo!(),
            Value::Record(..) => todo!(),
            Value::ArrayComprehension(..) => todo!(),
            Value::SetComprehension(..) => todo!(),
            Value::RecordComprehension(..) => todo!(),
            Value::Sequence(exprs) => {
                let guard = self.di.push_block_scope(expression.span);
                let mut exprs = exprs.iter();
                self.compile_expression(scope, target, exprs.next().unwrap())?;
                for expr in exprs {
                    self.trilogy_value_destroy(target);
                    self.compile_expression(scope, target, expr)?;
                }
                std::mem::drop(guard);
            }
            Value::Application(app) => self.compile_application(scope, target, app)?,
            Value::Builtin(val) => self.reference_builtin(scope, target, *val),
            Value::Reference(val) => self.compile_reference(scope, target, val),
            Value::ModuleAccess(access) => {
                self.compile_module_access(scope, target, &access.0, &access.1)
            }
            Value::IfElse(if_else) => self.compile_if_else(scope, target, if_else)?,
            Value::Assignment(assign) => self.compile_assignment(scope, target, assign)?,
            Value::While(..) => todo!(),
            Value::For(..) => todo!(),
            Value::Let(expr) => self.compile_let(scope, target, expr)?,
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

        if let Some(prev) = prev {
            self.builder.set_current_debug_location(prev);
        }

        Some(())
    }

    fn compile_array(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        pack: &ir::Pack,
    ) -> Option<()> {
        let array_value = self.trilogy_array_init_cap(target, pack.values.len(), "arr");
        let temporary = self.allocate_value("arr.el");
        for element in &pack.values {
            self.compile_expression(scope, temporary, &element.expression)?;
            if element.is_spread {
                self.trilogy_array_append(array_value, temporary);
            } else {
                self.trilogy_array_push(array_value, temporary);
            }
            self.trilogy_value_destroy(temporary);
        }
        Some(())
    }

    fn reference_builtin(
        &self,
        _scope: &mut Scope<'ctx>,
        _target: PointerValue<'ctx>,
        builtin: Builtin,
    ) {
        todo!("reference {:?}", builtin);
    }

    fn compile_let(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        decl: &ir::Let,
    ) -> Option<()> {
        match &decl.query.value {
            QueryValue::Direct(unif) => {
                let on_fail = self.context.append_basic_block(scope.function, "");
                let cont = self.builder.get_insert_block().unwrap();
                self.builder.position_at_end(on_fail);
                _ = self.internal_panic(
                    self.embed_c_string("unexpected end of execution (no match in declaration)\n"),
                );
                self.builder.position_at_end(cont);

                let value = self.allocate_expression(scope, &unif.expression, "")?;
                self.compile_pattern_match(scope, &unif.pattern, value, on_fail)?;
                self.trilogy_value_destroy(value);
                self.compile_expression(scope, target, &decl.body)?;
            }
            _ => todo!("non-deterministic branching "),
        }
        Some(())
    }

    fn compile_application(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        application: &ir::Application,
    ) -> Option<()> {
        match &application.function.value {
            Value::Builtin(builtin) if builtin.is_unary() => {
                return self.compile_apply_builtin(scope, target, *builtin, &application.argument)
            }
            Value::Application(app) => match &app.function.value {
                Value::Builtin(builtin) if builtin.is_binary() => {
                    return self.compile_apply_binary(
                        scope,
                        target,
                        *builtin,
                        &app.argument,
                        &application.argument,
                    )
                }
                _ => {}
            },
            _ => {}
        };
        let function = self.allocate_expression(scope, &application.function, "")?;
        match &application.argument.value {
            // Procedure application
            Value::Pack(pack) => {
                let arguments = pack
                    .values
                    .iter()
                    .map(|val| {
                        assert!(!val.is_spread);
                        self.allocate_expression(scope, &val.expression, "")
                    })
                    .collect::<Option<Vec<_>>>()?;
                self.call_procedure(
                    scope,
                    target,
                    function,
                    &arguments
                        .iter()
                        .map(|arg| (*arg).into())
                        .collect::<Vec<_>>(),
                );
                for argument in arguments {
                    self.trilogy_value_destroy(argument);
                }
            }
            // Function application
            _ => {
                let argument = self.allocate_expression(scope, &application.argument, "")?;
                self.apply_function(scope, target, function, argument.into());
                self.trilogy_value_destroy(argument);
            }
        }
        self.trilogy_value_destroy(function);
        Some(())
    }

    fn compile_module_access(
        &self,
        _scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        module_ref: &ir::Expression,
        ident: &syntax::Identifier,
    ) {
        // Possibly a static module reference, which we can support very easily and efficiently
        if let Value::Reference(name) = &module_ref.value {
            if let Some(Head::Module(name)) = self.globals.get(&name.id) {
                let declared = self
                    .module
                    .get_function(&format!("{}::{}", name, ident.as_ref()))
                    .unwrap();
                self.call_procedure_direct(target, declared, &[]);
                return;
            }
        }

        todo!()
    }

    fn compile_apply_builtin(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        builtin: Builtin,
        expression: &ir::Expression,
    ) -> Option<()> {
        match builtin {
            Builtin::Return => {
                self.compile_expression(scope, scope.sret(), expression)?;
                self.builder
                    .build_unconditional_branch(scope.cleanup.unwrap())
                    .unwrap();
                None
            }
            Builtin::Exit => {
                self.compile_expression(scope, target, expression)?;
                _ = self.exit(target);
                None
            }
            Builtin::Typeof => {
                let argument = self.allocate_expression(scope, expression, "")?;
                let tag = self.get_tag(argument);
                let raw_atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                self.trilogy_atom_init(target, raw_atom);
                self.trilogy_value_destroy(argument);
                Some(())
            }
            _ => todo!(),
        }
    }

    fn compile_apply_binary(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        builtin: Builtin,
        lhs: &ir::Expression,
        rhs: &ir::Expression,
    ) -> Option<()> {
        match builtin {
            Builtin::StructuralEquality => {
                let lhs = self.allocate_expression(scope, lhs, "seq.lhs")?;
                let rhs = self.allocate_expression(scope, rhs, "seq.rhs")?;
                self.structural_eq(target, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(())
            }
            Builtin::ReferenceEquality => {
                let lhs = self.allocate_expression(scope, lhs, "req.lhs")?;
                let rhs = self.allocate_expression(scope, rhs, "req.rhs")?;
                self.referential_eq(target, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(())
            }
            _ => todo!(),
        }
    }

    fn compile_assignment(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        assign: &ir::Assignment,
    ) -> Option<()> {
        match &assign.lhs.value {
            Value::Reference(variable) => {
                self.compile_expression(scope, target, &assign.rhs)?;
                let pointer = *scope.variables.get(&variable.id).unwrap();
                self.trilogy_value_destroy(pointer);
                self.builder.build_store(pointer, target).unwrap();
                Some(())
            }
            Value::Application(..) => todo!(),
            _ => panic!("invalid lvalue in assignment"),
        }
    }

    fn compile_reference(
        &self,
        scope: &Scope<'ctx>,
        target: PointerValue<'ctx>,
        identifier: &ir::Identifier,
    ) {
        if let Some(variable) = scope.variables.get(&identifier.id) {
            self.trilogy_value_clone_into(target, *variable);
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
                    self.call_procedure_direct(target, function, &[])
                }
                _ => todo!(),
            }
        }
    }

    fn compile_if_else(
        &self,
        scope: &mut Scope<'ctx>,
        target: PointerValue<'ctx>,
        if_else: &ir::IfElse,
    ) -> Option<()> {
        let condition = self.allocate_expression(scope, &if_else.condition, "if.cond")?;
        let if_true = self.context.append_basic_block(scope.function, "if.true");
        let if_false = self.context.append_basic_block(scope.function, "if.false");
        let if_cont = self.context.append_basic_block(scope.function, "if.cont");
        let cond_bool = self.trilogy_boolean_untag(condition, "");
        self.trilogy_value_destroy(condition);
        self.builder
            .build_conditional_branch(cond_bool, if_true, if_false)
            .unwrap();

        self.builder.position_at_end(if_true);
        let when_true = self.compile_expression(scope, target, &if_else.when_true);
        if when_true.is_some() {
            self.builder.build_unconditional_branch(if_cont).unwrap();
        }

        self.builder.position_at_end(if_false);
        let when_false = self.compile_expression(scope, target, &if_else.when_false);
        if when_false.is_some() {
            self.builder.build_unconditional_branch(if_cont).unwrap();
        }

        self.builder.position_at_end(if_cont);
        when_false.or(when_true)
    }
}
