use crate::{codegen::Head, Codegen};
use inkwell::{
    debug_info::AsDIScope, llvm_sys::debuginfo::LLVMDIFlagPublic, module::Linkage,
    values::PointerValue,
};
use num::{ToPrimitive, Zero};
use trilogy_ir::ir::{self, Builtin, QueryValue, Value};
use trilogy_parser::syntax;

impl<'ctx> Codegen<'ctx> {
    #[must_use = "allocated value must be destroyed"]
    pub(crate) fn allocate_expression(
        &self,
        expression: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        let target = self.allocate_value(name);
        self.compile_expression(target, expression)?;
        Some(target)
    }

    #[must_use = "must acknowldge continuation of control flow"]
    pub(crate) fn compile_expression(
        &self,
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
            Value::Array(arr) => self.compile_array(target, arr)?,
            Value::Set(..) => todo!(),
            Value::Record(..) => todo!(),
            Value::ArrayComprehension(..) => todo!(),
            Value::SetComprehension(..) => todo!(),
            Value::RecordComprehension(..) => todo!(),
            Value::Sequence(seq) => {
                self.di.push_block_scope(expression.span);
                let res = self.compile_sequence(target, seq);
                self.di.pop_scope();
                res?;
            }
            Value::Application(app) => self.compile_application(target, app)?,
            Value::Builtin(val) => self.reference_builtin(target, *val),
            Value::Reference(val) => self.compile_reference(target, val),
            Value::ModuleAccess(access) => self.compile_module_access(target, &access.0, &access.1),
            Value::IfElse(if_else) => self.compile_if_else(target, if_else)?,
            Value::Assignment(assign) => self.compile_assignment(target, assign)?,
            Value::While(..) => todo!(),
            Value::For(..) => todo!(),
            Value::Let(expr) => self.compile_let(target, expr)?,
            Value::Match(..) => todo!(),
            Value::Assert(..) => todo!(),
            Value::Fn(..) => todo!(),
            Value::Do(closure) => self.compile_do(target, closure),
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

    fn compile_sequence(&self, target: PointerValue<'ctx>, seq: &[ir::Expression]) -> Option<()> {
        let mut exprs = seq.iter();
        self.compile_expression(target, exprs.next().unwrap())?;
        for expr in exprs {
            self.trilogy_value_destroy(target);
            self.compile_expression(target, expr)?;
        }
        Some(())
    }

    fn compile_array(&self, target: PointerValue<'ctx>, pack: &ir::Pack) -> Option<()> {
        let array_value = self.trilogy_array_init_cap(target, pack.values.len(), "arr");
        let temporary = self.allocate_value("arr.el");
        for element in &pack.values {
            self.compile_expression(temporary, &element.expression)?;
            if element.is_spread {
                self.trilogy_array_append(array_value, temporary);
            } else {
                self.trilogy_array_push(array_value, temporary);
            }
            self.trilogy_value_destroy(temporary);
        }
        Some(())
    }

    fn reference_builtin(&self, _target: PointerValue<'ctx>, builtin: Builtin) {
        todo!("reference {:?}", builtin);
    }

    fn compile_let(&self, target: PointerValue<'ctx>, decl: &ir::Let) -> Option<()> {
        match &decl.query.value {
            QueryValue::Direct(unif) => {
                let on_fail = self.context.append_basic_block(self.get_function(), "");
                let cont = self.builder.get_insert_block().unwrap();
                self.builder.position_at_end(on_fail);
                _ = self.internal_panic(
                    self.embed_c_string("unexpected end of execution (no match in declaration)\n"),
                );
                self.builder.position_at_end(cont);

                let value = self.allocate_expression(&unif.expression, "")?;
                self.compile_pattern_match(&unif.pattern, value, on_fail)?;
                self.trilogy_value_destroy(value);
                self.compile_expression(target, &decl.body)?;
            }
            _ => todo!("non-deterministic branching "),
        }
        Some(())
    }

    fn compile_application(
        &self,
        target: PointerValue<'ctx>,
        application: &ir::Application,
    ) -> Option<()> {
        match &application.function.value {
            Value::Builtin(builtin) if builtin.is_unary() => {
                return self.compile_apply_builtin(target, *builtin, &application.argument)
            }
            Value::Application(app) => match &app.function.value {
                Value::Builtin(builtin) if builtin.is_binary() => {
                    return self.compile_apply_binary(
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
        let function = self.allocate_expression(&application.function, "")?;
        match &application.argument.value {
            // Procedure application
            Value::Pack(pack) => {
                let arguments = pack
                    .values
                    .iter()
                    .map(|val| {
                        assert!(!val.is_spread);
                        self.allocate_expression(&val.expression, "")
                    })
                    .collect::<Option<Vec<_>>>()?;
                self.call_procedure(
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
                let argument = self.allocate_expression(&application.argument, "")?;
                self.apply_function(function, argument.into());
                self.trilogy_value_destroy(argument);
            }
        }
        self.trilogy_value_destroy(function);
        Some(())
    }

    fn compile_module_access(
        &self,
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
                self.call_internal(target, declared, &[]);
                return;
            }
        }

        todo!()
    }

    fn compile_apply_builtin(
        &self,
        target: PointerValue<'ctx>,
        builtin: Builtin,
        expression: &ir::Expression,
    ) -> Option<()> {
        match builtin {
            Builtin::Return => {
                let result = self.allocate_expression(expression, "")?;
                let return_cont = self
                    .get_function()
                    .get_first_param()
                    .unwrap()
                    .into_pointer_value();
                let return_call = self.call_continuation(return_cont, result.into());
                self.set_returned(return_call);
                None
            }
            Builtin::Exit => {
                let result = self.allocate_expression(expression, "")?;
                _ = self.exit(result);
                None
            }
            Builtin::Typeof => {
                let argument = self.allocate_expression(expression, "")?;
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
        target: PointerValue<'ctx>,
        builtin: Builtin,
        lhs: &ir::Expression,
        rhs: &ir::Expression,
    ) -> Option<()> {
        match builtin {
            Builtin::StructuralEquality => {
                let lhs = self.allocate_expression(lhs, "seq.lhs")?;
                let rhs = self.allocate_expression(rhs, "seq.rhs")?;
                self.structural_eq(target, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(())
            }
            Builtin::ReferenceEquality => {
                let lhs = self.allocate_expression(lhs, "req.lhs")?;
                let rhs = self.allocate_expression(rhs, "req.rhs")?;
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
        target: PointerValue<'ctx>,
        assign: &ir::Assignment,
    ) -> Option<()> {
        match &assign.lhs.value {
            Value::Reference(variable) => {
                self.compile_expression(target, &assign.rhs)?;
                let pointer = self.get_variable(&variable.id).unwrap();
                self.trilogy_value_destroy(pointer);
                self.trilogy_value_clone_into(pointer, target);
                Some(())
            }
            Value::Application(..) => todo!(),
            _ => panic!("invalid lvalue in assignment"),
        }
    }

    fn compile_reference(&self, target: PointerValue<'ctx>, identifier: &ir::Identifier) {
        if let Some(variable) = self.get_variable(&identifier.id) {
            self.trilogy_value_clone_into(target, variable);
        } else {
            let name = identifier.id.name().unwrap();
            match self
                .globals
                .get(&identifier.id)
                .expect("Unresolved variable")
            {
                Head::Constant | Head::Procedure => {
                    let global_name =
                        format!("{}::{name}", self.module.get_name().to_str().unwrap());
                    let function = self.module.get_function(&global_name).unwrap();
                    self.call_internal(target, function, &[])
                }
                _ => todo!(),
            }
        }
    }

    fn compile_if_else(&self, target: PointerValue<'ctx>, if_else: &ir::IfElse) -> Option<()> {
        let condition = self.allocate_expression(&if_else.condition, "if.cond")?;
        let function = self.get_function();
        let if_true = self.context.append_basic_block(function, "if.true");
        let if_false = self.context.append_basic_block(function, "if.false");
        let if_cont = self.context.append_basic_block(function, "if.cont");
        let cond_bool = self.trilogy_boolean_untag(condition, "");
        self.trilogy_value_destroy(condition);
        self.builder
            .build_conditional_branch(cond_bool, if_true, if_false)
            .unwrap();

        self.builder.position_at_end(if_true);
        let when_true = self.compile_expression(target, &if_else.when_true);
        if when_true.is_some() {
            self.builder.build_unconditional_branch(if_cont).unwrap();
        }

        self.builder.position_at_end(if_false);
        let when_false = self.compile_expression(target, &if_else.when_false);
        if when_false.is_some() {
            self.builder.build_unconditional_branch(if_cont).unwrap();
        }

        self.builder.position_at_end(if_cont);
        when_false.or(when_true)
    }

    fn compile_do(&self, target: PointerValue<'ctx>, procedure: &ir::Procedure) {
        let function = self.get_function();
        let current = function.get_name().to_str().unwrap();
        let name = format!("{current}<do@{}>", procedure.span);
        let arity = procedure.parameters.len();

        let closure_codegen = self.inner();
        let procedure_scope = closure_codegen.di.builder.create_function(
            closure_codegen.di.unit.get_file().as_debug_info_scope(),
            &name,
            None,
            closure_codegen.di.unit.get_file(),
            procedure.span.start().line as u32 + 1,
            closure_codegen.di.closure_di_type(arity),
            false,
            true,
            procedure.span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );

        let function = closure_codegen.module.add_function(
            &name,
            closure_codegen.procedure_type(arity, true),
            Some(Linkage::Internal),
        );
        function.set_subprogram(procedure_scope);
        closure_codegen.compile_procedure_body(function, procedure);
        closure_codegen.close_as_do(target, function, arity);
    }
}

// TODO: all expressions must execute in CPS mode; a continuation is captured at certain points
// 1. Any call to a function or procedure
//      To enable the capture of `return`, all calls never return and instead go to a callback
//      The return closure is carried through the whole continuation of a procedure; implicitly in the context
// 2. Any non-deterministic `let`
//      An `or` is executed with each side on a separate execution
//      An `in` is executed with each element on a separate execution
//      A `rule` is executed with a execution spawned for each possible binding
//
//      In any case, the `end` keyword is implemented as a continuation into the runtime, created at this point,
//      that takes care of starting the next execution, or terminating the program.
// 3. Any branch point (`if`, `match`)
//      Not required for the branch itself, but required for the reconvergence
//      Technically only required if either branch diverges, but we can simplify implementation by always making a continuation
// 4. Any `when` or `yield`
//      Capture at `when` for `cancel`
//      Capture at `yield` for `resume`
// 5. Any `for` or `while`
//      Capture the exit of the loop for `break`
//      Capture the entry of the loop for `continue`
//
// This manifests as each expression being compiled as having two possibly "targets"
// 1. A pointer into which to save the evaluation result (as it is now)
// 2. A continuation into which to call with the result (but at the time of compilation, that continuation is not known)
//
// Since only the "compiler" of the expression knows which place that is, the expression compilation must return its
// value via an LLVM SSA register (e.g. an `Option<StructValue<'ctx>>` directly?); and the caller can deal with that.
// This is different than my previous two approaches (return an `Option<PointerValue<'ctx>>` controlling both data flow
// and control flow, and return an `Option<()>` controlling only the control flow), which did not work as the expression
// itself dictates neither the data flow nor control flow, and should not concern itself with those details.
//
//      Actually, maybe that's wrong, and it SHOULD be a pointer value, and we go back to the expression allocating its
//      own result, if any...
//
// The naive approach: always assume every variable is captured, and basically don't use the stack at all.
// Kind of easy to build...
//
// Slightly better: run codegen for "the rest" of this continuation, compute the captures, and reconstruct the context
// The challenge: IR is not CPS, so each expression does not know its continuation at time of codegen;
//      1. Run a CPS conversion pass between IR and LLVM. A lot of work, but reliable
//      2. Double-traverse during LLVM pass; go down, write code assuming a context, go back up and rebuild it. Messy, but easy, if it works.
//          Maintain a list of all parent nodes in the CPS graph, and every time a variable or keyword is referenced
//          revisit those nodes to add a capture to that variable, if not already.
//
// I think we're going with (2) here...
