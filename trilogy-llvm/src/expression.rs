use crate::{codegen::Head, Codegen};
use inkwell::{
    debug_info::AsDIScope,
    llvm_sys::debuginfo::LLVMDIFlagPublic,
    module::Linkage,
    values::{BasicValue, PointerValue},
};
use num::{ToPrimitive, Zero};
use trilogy_ir::ir::{self, Builtin, QueryValue, Value};
use trilogy_parser::syntax;

impl<'ctx> Codegen<'ctx> {
    #[must_use = "must acknowldge continuation of control flow"]
    pub(crate) fn compile_expression(
        &self,
        expression: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        let prev = self.set_span(expression.span);

        let result = match &expression.value {
            Value::Unit => {
                let val = self.allocate_value(name);
                self.builder.build_store(val, self.unit_const()).unwrap();
                Some(val)
            }
            Value::Boolean(b) => {
                let val = self.allocate_value(name);
                self.builder.build_store(val, self.bool_const(*b)).unwrap();
                Some(val)
            }
            Value::Atom(atom) => {
                let val = self.allocate_value(name);
                self.builder
                    .build_store(val, self.atom_const(atom.to_owned()))
                    .unwrap();
                Some(val)
            }
            Value::Character(ch) => {
                let val = self.allocate_value(name);
                self.builder.build_store(val, self.char_const(*ch)).unwrap();
                Some(val)
            }
            Value::String(s) => {
                let val = self.allocate_value(name);
                self.string_const(val, s);
                Some(val)
            }
            Value::Number(num) => {
                if num.value().im.is_zero() && num.value().re.is_integer() {
                    if let Some(int) = num.value().re.to_i64() {
                        let val = self.allocate_value(name);
                        self.builder.build_store(val, self.int_const(int)).unwrap();
                        Some(val)
                    } else {
                        todo!("Support large integers")
                    }
                } else {
                    todo!("Support non-integers")
                }
            }
            Value::Bits(b) => {
                let val = self.allocate_value(name);
                self.bits_const(val, b);
                Some(val)
            }
            Value::Array(arr) => self.compile_array(arr, name),
            Value::Set(..) => todo!(),
            Value::Record(..) => todo!(),
            Value::ArrayComprehension(..) => todo!(),
            Value::SetComprehension(..) => todo!(),
            Value::RecordComprehension(..) => todo!(),
            Value::Sequence(seq) => {
                self.di.push_block_scope(expression.span);
                let res = self.compile_sequence(seq, name);
                self.di.pop_scope();
                res
            }
            Value::Application(app) => self.compile_application(app, name),
            Value::Builtin(val) => Some(self.reference_builtin(*val, name)),
            Value::Reference(val) => Some(self.compile_reference(val, name)),
            Value::ModuleAccess(access) => {
                Some(self.compile_module_access(&access.0, &access.1, name))
            }
            Value::IfElse(if_else) => self.compile_if_else(if_else, name),
            Value::Assignment(assign) => self.compile_assignment(assign, name),
            Value::While(..) => todo!(),
            Value::For(..) => todo!(),
            Value::Let(expr) => self.compile_let(expr, name),
            Value::Match(..) => todo!(),
            Value::Assert(..) => todo!(),
            Value::Fn(..) => todo!(),
            Value::Do(closure) => Some(self.compile_do(closure, name)),
            Value::Qy(..) => todo!(),
            Value::Handled(..) => todo!(),
            Value::End => {
                self.compile_end();
                None
            }
            Value::Pack(..) => panic!("arbitrary packs are not permitted"),
            Value::Mapping(..) => panic!("arbitrary mappings are not permitted"),
            Value::Conjunction(..) => panic!("conjunction not permitted in expression context"),
            Value::Disjunction(..) => panic!("disjunction not permitted in expression context"),
            Value::Wildcard => panic!("wildcard not permitted in expression context"),
            Value::Query(..) => panic!("query not permitted in expression context"),
        };

        if let Some(prev) = prev {
            self.overwrite_debug_location(prev);
        }

        result
    }

    fn compile_end(&self) {
        let end = self.get_end("");
        let alloca = self.allocate_value("");
        self.call_continuation(end, alloca);
    }

    fn compile_sequence(&self, seq: &[ir::Expression], name: &str) -> Option<PointerValue<'ctx>> {
        let mut exprs = seq.iter();
        let mut value = self.compile_expression(exprs.next().unwrap(), name)?;
        for expr in exprs {
            self.trilogy_value_destroy(value);
            value = self.compile_expression(expr, name)?;
        }
        Some(value)
    }

    fn compile_array(&self, pack: &ir::Pack, name: &str) -> Option<PointerValue<'ctx>> {
        let target = self.allocate_value(name);
        let array_value = self.trilogy_array_init_cap(target, pack.values.len(), "arr");
        for element in &pack.values {
            let temporary = self.compile_expression(&element.expression, "arr.el")?;
            if element.is_spread {
                self.trilogy_array_append(array_value, temporary);
            } else {
                self.trilogy_array_push(array_value, temporary);
            }
            self.trilogy_value_destroy(temporary);
        }
        Some(target)
    }

    fn reference_builtin(&self, builtin: Builtin, name: &str) -> PointerValue<'ctx> {
        match builtin {
            Builtin::Return => self.get_return(name),
            _ => todo!(),
        }
    }

    fn compile_let(&self, decl: &ir::Let, name: &str) -> Option<PointerValue<'ctx>> {
        match &decl.query.value {
            QueryValue::Direct(unif) if decl.query.is_once() => {
                let on_fail = self.context.append_basic_block(self.get_function(), "");
                let cont = self.builder.get_insert_block().unwrap();
                self.builder.position_at_end(on_fail);
                _ = self.internal_panic(
                    self.embed_c_string("unexpected end of execution (no match in declaration)\n"),
                );
                self.builder.position_at_end(cont);

                let value = self.compile_expression(&unif.expression, "")?;
                self.compile_pattern_match(&unif.pattern, value)?;
                self.trilogy_value_destroy(value);
                self.compile_expression(&decl.body, name)
            }
            _ => todo!("non-deterministic branching"),
        }
    }

    fn compile_application(
        &self,
        application: &ir::Application,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        match &application.function.value {
            Value::Builtin(builtin) if builtin.is_unary() => {
                return self.compile_apply_builtin(*builtin, &application.argument, name)
            }
            Value::Application(app) => match &app.function.value {
                Value::Builtin(builtin) if builtin.is_binary() => {
                    return self.compile_apply_binary(
                        *builtin,
                        &app.argument,
                        &application.argument,
                        name,
                    )
                }
                _ => {}
            },
            _ => {}
        };
        let function = self.compile_expression(&application.function, "")?;
        self.bind_temporary(function);
        match &application.argument.value {
            // Procedure application
            Value::Pack(pack) => {
                let mut arguments = Vec::with_capacity(pack.values.len());
                for val in pack.values.iter() {
                    assert!(
                        !val.is_spread,
                        "a spread is not permitted in procedure argument lists"
                    );
                    let param = self.compile_expression(&val.expression, "")?;
                    self.bind_temporary(param);
                    arguments.push(param);
                }
                let function = self.use_temporary(function).unwrap().ptr();
                for arg in arguments.iter_mut() {
                    *arg = self.use_temporary(*arg).unwrap().ptr();
                }
                Some(self.call_procedure(function, &arguments))
            }
            // Function application
            _ => {
                let argument = self.compile_expression(&application.argument, "")?;
                Some(self.apply_function(function, argument))
            }
        }
    }

    fn compile_module_access(
        &self,
        module_ref: &ir::Expression,
        ident: &syntax::Identifier,
        name: &str,
    ) -> PointerValue<'ctx> {
        // Possibly a static module reference, which we can support very easily and efficiently
        if let Value::Reference(module) = &module_ref.value {
            if let Some(Head::Module(module)) = self.globals.get(&module.id) {
                let target = self.allocate_value(name);
                let declared = self
                    .module
                    .get_function(&format!("{}::{}", module, ident.as_ref()))
                    .unwrap();
                self.call_internal(target, declared, &[]);
                return target;
            }
        }

        todo!()
    }

    fn compile_apply_builtin(
        &self,
        builtin: Builtin,
        expression: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        match builtin {
            Builtin::Return => {
                let result = self.compile_expression(expression, name)?;
                let return_cont = self.get_return("");
                self.call_continuation(return_cont, result);
                None
            }
            Builtin::Exit => {
                let result = self.compile_expression(expression, name)?;
                _ = self.exit(result);
                None
            }
            Builtin::Typeof => {
                let out = self.allocate_value(name);
                let argument = self.compile_expression(expression, "")?;
                let tag = self.get_tag(argument);
                let raw_atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                self.trilogy_atom_init(out, raw_atom);
                self.trilogy_value_destroy(argument);
                Some(out)
            }
            _ => todo!(),
        }
    }

    fn compile_apply_binary(
        &self,
        builtin: Builtin,
        lhs: &ir::Expression,
        rhs: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        match builtin {
            Builtin::StructuralEquality => {
                let out = self.allocate_value(name);
                let lhs = self.compile_expression(lhs, "seq.lhs")?;
                let rhs = self.compile_expression(rhs, "seq.rhs")?;
                self.structural_eq(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::ReferenceEquality => {
                let out = self.allocate_value(name);
                let lhs = self.compile_expression(lhs, "req.lhs")?;
                let rhs = self.compile_expression(rhs, "req.rhs")?;
                self.referential_eq(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Access => {
                let out = self.allocate_value(name);
                let lhs = self.compile_expression(lhs, "acc.c")?;
                let rhs = self.compile_expression(rhs, "acc.i")?;
                self.member_access(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Cons => {
                let out = self.allocate_value(name);
                let lhs = self.compile_expression(lhs, "cons.lhs")?;
                let rhs = self.compile_expression(rhs, "cons.rhs")?;
                self.trilogy_tuple_init_new(out, lhs, rhs);
                Some(out)
            }
            Builtin::Construct => {
                let out = self.allocate_value(name);
                let lhs = self.compile_expression(lhs, "struct.val")?;
                let rhs = self.compile_expression(rhs, "")?;
                let tag = self.trilogy_atom_untag(rhs, "struct.tag");
                self.trilogy_value_destroy(rhs);
                self.trilogy_struct_init_new(out, tag, lhs);
                Some(out)
            }
            Builtin::Glue => {
                let out = self.allocate_value(name);
                let lhs = self.compile_expression(lhs, "glue.lhs")?;
                let rhs = self.compile_expression(rhs, "glue.rhs")?;
                self.glue(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            _ => todo!(),
        }
    }

    fn compile_assignment(
        &self,
        assign: &ir::Assignment,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        match &assign.lhs.value {
            Value::Reference(variable) => {
                let value = self.compile_expression(&assign.rhs, name)?;
                let variable = self.get_variable(&variable.id).unwrap();
                self.trilogy_value_destroy(variable.ptr());
                self.trilogy_value_clone_into(variable.ptr(), value);
                Some(value)
            }
            Value::Application(..) => todo!(),
            _ => panic!("invalid lvalue in assignment"),
        }
    }

    fn compile_reference(&self, identifier: &ir::Identifier, name: &str) -> PointerValue<'ctx> {
        if let Some(variable) = self.get_variable(&identifier.id) {
            let target = self.allocate_value(name);
            self.trilogy_value_clone_into(target, variable.ptr());
            target
        } else {
            let ident = identifier.id.name().unwrap();
            match self
                .globals
                .get(&identifier.id)
                .expect("Unresolved variable")
            {
                Head::Constant | Head::Procedure => {
                    let target = self.allocate_value(name);
                    let global_name =
                        format!("{}::{ident}", self.module.get_name().to_str().unwrap());
                    let function = self.module.get_function(&global_name).unwrap();
                    self.call_internal(target, function, &[]);
                    target
                }
                _ => todo!(),
            }
        }
    }

    fn compile_if_else(&self, if_else: &ir::IfElse, name: &str) -> Option<PointerValue<'ctx>> {
        let condition = self.compile_expression(&if_else.condition, "if.cond")?;

        let function = self.get_function();
        let if_true_block = self.context.append_basic_block(function, "if.true");
        let if_false_block = self.context.append_basic_block(function, "if.false");
        let if_true_function = self.add_continuation("if.true");
        let if_false_function = self.add_continuation("if.false");
        let merge_to_function = self.add_continuation("if.cont");

        let cond_bool = self.trilogy_boolean_untag(condition, "if.cond");
        self.trilogy_value_destroy(condition);
        self.builder
            .build_conditional_branch(cond_bool, if_true_block, if_false_block)
            .unwrap();

        let brancher = self.branch();
        let mut merger = self.merger();

        self.builder.position_at_end(if_true_block);
        let continue_to =
            self.continue_to(if_true_function, self.allocate_const(self.unit_const(), ""));
        self.close_from(
            &brancher,
            continue_to,
            self.builder.get_current_debug_location().unwrap(),
        );

        let if_true_entry = self.context.append_basic_block(if_true_function, "entry");
        self.builder.position_at_end(if_true_entry);
        self.transfer_debug_info(if_true_function);
        let when_true = self.compile_expression(&if_else.when_true, name);

        if let Some(value) = when_true {
            let continue_to = self.continue_to(merge_to_function, value);
            self.merge_into(
                &mut merger,
                continue_to,
                self.builder.get_current_debug_location().unwrap(),
            );
        }

        self.builder.position_at_end(if_false_block);
        // NOTE: This is a bit unfortunate, as it creates another copy of the same debug info block hierarchy
        // up to this if. Would be nice if I could compile the branches, then the functions of those branches,
        // instead of like now when it's doing the true function in between the branches.
        self.transfer_debug_info(function);
        let continue_to = self.continue_to(
            if_false_function,
            self.allocate_const(self.unit_const(), ""),
        );
        self.close_from(
            &brancher,
            continue_to,
            self.builder.get_current_debug_location().unwrap(),
        );

        let if_false_entry = self.context.append_basic_block(if_false_function, "entry");
        self.builder.position_at_end(if_false_entry);
        self.transfer_debug_info(if_false_function);
        let when_false = self.compile_expression(&if_else.when_false, name);

        if let Some(value) = when_false {
            let continue_to = self.continue_to(merge_to_function, value);
            self.merge_into(
                &mut merger,
                continue_to,
                self.builder.get_current_debug_location().unwrap(),
            );
        }

        if when_true.is_some() || when_false.is_some() {
            self.merge_branch(brancher, merger);
            let entry = self.context.append_basic_block(merge_to_function, "entry");
            self.builder.position_at_end(entry);
            self.transfer_debug_info(merge_to_function);
            Some(self.get_continuation(""))
        } else {
            None
        }
    }

    fn compile_do(&self, procedure: &ir::Procedure, name: &str) -> PointerValue<'ctx> {
        let (current, _) = self.get_current_definition();
        let function_name = format!("{current}<do@{}>", procedure.span);
        let arity = procedure.parameters.len();
        let function = self.module.add_function(
            &function_name,
            self.procedure_type(arity, true),
            Some(Linkage::Internal),
        );

        let target = self.allocate_value(name);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        let brancher = self.branch();
        self.trilogy_callable_init_do(
            target,
            arity,
            closure,
            function.as_global_value().as_pointer_value(),
        );
        let here = self.builder.get_insert_block().unwrap();

        self.capture_from(
            &brancher,
            closure.as_instruction_value().unwrap(),
            self.builder.get_current_debug_location().unwrap(),
        );

        let procedure_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            &function_name,
            None,
            self.di.unit.get_file(),
            procedure.span.start().line as u32 + 1,
            self.di.closure_di_type(arity),
            true,
            true,
            procedure.span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        function.set_subprogram(procedure_scope);
        self.compile_procedure_body(function, procedure);

        self.builder.position_at_end(here);
        self.continue_from(&brancher);
        target
    }
}
