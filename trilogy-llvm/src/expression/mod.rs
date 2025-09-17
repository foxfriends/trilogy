use crate::codegen::{Codegen, Global, Head, Merger, Variable};
use inkwell::debug_info::AsDIScope;
use inkwell::llvm_sys::debuginfo::LLVMDIFlagPublic;
use inkwell::module::Linkage;
use inkwell::values::{BasicValue, PointerValue};
use trilogy_ir::ir::{self, Builtin, QueryValue, Value};
use trilogy_ir::visitor::Bindings;
use trilogy_parser::syntax;

mod builtin;

impl<'ctx> Codegen<'ctx> {
    #[must_use = "must acknowldge continuation of control flow"]
    pub(crate) fn compile_expression(
        &self,
        expression: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        let prev = self.set_span(expression.span);

        let result = match &expression.value {
            Value::Unit => Some(self.allocate_const(self.unit_const(), name)),
            Value::Boolean(b) => Some(self.allocate_const(self.bool_const(*b), name)),
            Value::Atom(atom) => Some(self.allocate_const(self.atom_const(atom.to_owned()), name)),
            Value::Character(ch) => Some(self.allocate_const(self.char_const(*ch), name)),
            Value::String(s) => {
                let val = self.allocate_value(name);
                self.string_const(val, s);
                Some(val)
            }
            Value::Number(num) => {
                let val = self.allocate_value(name);
                self.number_const(val, num);
                Some(val)
            }
            Value::Bits(b) => {
                let val = self.allocate_value(name);
                self.bits_const(val, b);
                Some(val)
            }
            Value::Array(arr) => self.compile_array(arr, name),
            Value::Set(..) => todo!(),
            Value::Record(record) => self.compile_record(record, name),
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
            Value::ModuleAccess(access) => self.compile_module_access(&access.0, &access.1, name),
            Value::IfElse(if_else) => self.compile_if_else(if_else, name),
            Value::Assignment(assign) => self.compile_assignment(assign, name),
            Value::While(expr) => self.compile_while(expr, name),
            Value::For(expr) => self.compile_for(expr, name),
            Value::Let(expr) => self.compile_let(expr, name),
            Value::Match(expr) => self.compile_match(expr, name),
            Value::Assert(assertion) => self.compile_assertion(assertion, name),
            Value::Fn(closure) => Some(self.compile_fn(closure, name)),
            Value::Do(closure) => Some(self.compile_do(closure, name)),
            Value::Qy(closure) => Some(self.compile_qy(closure, name)),
            Value::Handled(handled) => self.compile_handled(handled, name),
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
        self.void_call_continuation(self.get_end(""));
    }

    fn compile_while(&self, expr: &ir::While, name: &str) -> Option<PointerValue<'ctx>> {
        let continue_function = self.add_continuation("while");
        let break_function = self.add_continuation("while.done");

        let (break_continuation, break_continuation_point) =
            self.capture_current_continuation_as_break(break_function, "");
        self.continue_in_loop(continue_function, break_continuation);
        self.begin_next_function(continue_function);
        // TODO: within the condition of a loop, `break` keyword should refer to the parent scope's break,
        // but here it refers to the child.
        //
        // Maybe solve this just by disallowing break keyword in loop condition entirely, and require
        // explicit usage of a bound break variable?
        let condition = self.compile_expression(&expr.condition, "while.condition")?;
        let bool_value = self.trilogy_boolean_untag(condition, name);
        self.trilogy_value_destroy(condition);

        let then_block = self
            .context
            .append_basic_block(self.get_function(), "while.then");
        let else_block = self
            .context
            .append_basic_block(self.get_function(), "while.else");
        self.builder
            .build_conditional_branch(bool_value, then_block, else_block)
            .unwrap();
        let body_cp = self.branch_continuation_point();
        let snapshot = self.snapshot_function_context();

        self.builder.position_at_end(else_block);
        let break_continuation = self.get_break("break");
        self.call_known_continuation(
            break_continuation,
            self.allocate_const(self.unit_const(), ""),
        );

        self.builder.position_at_end(then_block);
        self.restore_function_context(snapshot);
        self.become_continuation_point(body_cp);
        if let Some(result) = self.compile_expression(&expr.body, name) {
            self.call_continue(result, "");
        }

        self.become_continuation_point(break_continuation_point);
        self.begin_next_function(break_function);
        Some(self.get_continuation(name))
    }

    fn compile_for(&self, expr: &ir::Iterator, name: &str) -> Option<PointerValue<'ctx>> {
        let done_function = self.add_continuation("done");
        let (done_continuation, done_continuation_point) =
            self.capture_current_continuation_as_break(done_function, "for_break");
        let next_iteration = self.compile_iterator(&expr.query, done_continuation)?;
        self.bind_temporary(next_iteration);
        if let Some(value) = self.compile_expression(&expr.value, name) {
            let next_iteration = self.use_temporary(next_iteration).unwrap();
            self.call_known_continuation(next_iteration, value);
        }

        self.become_continuation_point(done_continuation_point);
        self.begin_next_function(done_function);
        // TODO: currently `for..else` is expecting this to return a boolean instead of a unit, but
        // that's not really right... the for should really somehow be a "fold" construct eventually,
        // returning neither unit or boolean.
        //
        // let [1, 2, 3] = from list = [] for vals(x) { [...list, x] }
        // let [1, 2, 3] = for vals(x) into list = [] { [...list, x] }
        //
        // The `for..else` will need to just be transformed into a thing with a flag in it, at the
        // IR level (or source level and drop the feature).
        //
        // if !(for vals(x) into ok = false { true }) { for_else }
        Some(self.allocate_const(self.unit_const(), ""))
    }

    fn compile_handled(&self, handled: &ir::Handled, name: &str) -> Option<PointerValue<'ctx>> {
        let body_function = self.add_continuation("");
        let handler_function = self.add_continuation(name);

        // Prepare cancel continuation for after the handled section is complete.
        let cancel_to_function = self.add_continuation("when.cancel");
        let (cancel_to, cancel_to_continuation_point) =
            self.capture_current_continuation_as_cancel(cancel_to_function, "");

        // Construct yield continuation that continues into the handler itself.
        let cancel_to_clone = self.allocate_value("");
        self.trilogy_value_clone_into(cancel_to_clone, cancel_to);
        let (handler, handler_continuation_point) =
            self.capture_current_continuation_as_yield(handler_function, cancel_to_clone, "");

        // Then enter the handler, given the new yield and cancel to values`
        let body_closure = self.continue_in_handler(body_function, handler, cancel_to);
        self.end_continuation_point_as_close(body_closure);

        self.begin_next_function(body_function);
        let result = self.compile_expression(&handled.expression, name)?;

        let cancel_to = self.get_cancel("when.runoff");
        self.call_known_continuation(cancel_to, result);

        self.become_continuation_point(handler_continuation_point);
        self.begin_next_function(handler_function);
        self.compile_handlers(&handled.handlers);

        self.become_continuation_point(cancel_to_continuation_point);
        self.begin_next_function(cancel_to_function);
        Some(self.get_continuation(name))
    }

    fn compile_handlers(&self, handlers: &[ir::Handler]) {
        let effect = self.get_continuation("effect");
        self.bind_temporary(effect);

        for handler in handlers {
            let next_case_function = self.add_continuation("");
            let (go_to_next_case, next_case_cp) =
                self.capture_current_continuation(next_case_function, "when.next");
            let effect = self.use_temporary(effect).unwrap();
            if self
                .compile_pattern_match(&handler.pattern, effect, go_to_next_case)
                .is_none()
            {
                break;
            }
            let Some(guard_bool) = self.compile_expression(&handler.guard, "when.guard") else {
                self.become_continuation_point(next_case_cp);
                self.begin_next_function(next_case_function);
                continue;
            };
            let guard_flag = self.trilogy_boolean_untag(guard_bool, "");
            // NOTE: bool doesn't really need to be destroyed... but do it anyway
            self.trilogy_value_destroy(guard_bool);
            let body_block = self.context.append_basic_block(self.get_function(), "body");
            let next_block = self.context.append_basic_block(self.get_function(), "next");
            let body_cp = self.branch_continuation_point();
            self.builder
                .build_conditional_branch(guard_flag, body_block, next_block)
                .unwrap();
            let snapshot = self.snapshot_function_context();

            self.builder.position_at_end(next_block);
            let go_next = self.use_temporary(go_to_next_case).unwrap();
            self.void_call_continuation(go_next);

            self.builder.position_at_end(body_block);
            self.restore_function_context(snapshot);
            self.become_continuation_point(body_cp);
            if let Some(result) = self.compile_expression(&handler.body, "handler_result") {
                self.trilogy_value_destroy(result);
                self.void_call_continuation(self.get_end(""));
            }

            self.become_continuation_point(next_case_cp);
            self.begin_next_function(next_case_function);
        }

        let unreachable = self.builder.build_unreachable().unwrap();
        self.end_continuation_point_as_clean(unreachable);
    }

    fn compile_assertion(&self, assertion: &ir::Assert, name: &str) -> Option<PointerValue<'ctx>> {
        let expression = self.compile_expression(&assertion.assertion, name)?;

        let pass_cp = self.branch_continuation_point();
        let cond = self.trilogy_boolean_untag(expression, "");
        let pass = self
            .context
            .append_basic_block(self.get_function(), "assert.pass");
        let fail = self
            .context
            .append_basic_block(self.get_function(), "assert.fail");
        self.builder
            .build_conditional_branch(cond, pass, fail)
            .unwrap();
        let snapshot = self.snapshot_function_context();

        self.builder.position_at_end(fail);
        if let Some(msg) = self.compile_expression(&assertion.message, "assert.msg") {
            let panic = self.panic(msg);
            self.builder.build_unreachable().unwrap();
            self.end_continuation_point_as_clean(panic);
        }

        self.builder.position_at_end(pass);
        self.become_continuation_point(pass_cp);
        self.restore_function_context(snapshot);
        Some(expression)
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
        self.bind_temporary(target);
        self.trilogy_array_init_cap(target, pack.values.len(), "arr");
        for element in &pack.values {
            let temporary = self.compile_expression(&element.expression, "arr.el")?;
            let target = self.use_temporary(target).unwrap();
            let array_value = self.trilogy_array_assume(target, "");
            if element.is_spread {
                self.trilogy_array_append(array_value, temporary);
            } else {
                self.trilogy_array_push(array_value, temporary);
            }
        }
        let target = self.use_temporary(target).unwrap();
        Some(target)
    }

    fn compile_record(&self, pack: &ir::Pack, name: &str) -> Option<PointerValue<'ctx>> {
        let target = self.allocate_value(name);
        self.bind_temporary(target);
        self.trilogy_record_init_cap(target, pack.values.len(), "rec");
        for element in &pack.values {
            match &element.expression.value {
                ir::Value::Mapping(kv) => {
                    let key = self.compile_expression(&kv.0, "rec.k")?;
                    self.bind_temporary(key);
                    let value = self.compile_expression(&kv.1, "rec.v")?;
                    let record = self.use_temporary(target).unwrap();
                    let record = self.trilogy_record_assume(record, "");
                    let key = self.use_temporary(key).unwrap();
                    self.trilogy_record_insert(record, key, value);
                }
                _value if element.is_spread => todo!(),
                _ => panic!("record elements must be spread or mapping"),
            }
        }
        let target = self.use_temporary(target).unwrap();
        Some(target)
    }

    fn compile_let(&self, decl: &ir::Let, name: &str) -> Option<PointerValue<'ctx>> {
        match &decl.query.value {
            QueryValue::Direct(unif) if decl.query.is_once() => {
                // These variables are tricky... if they are invented only when initialized, they
                // are trivially cleared before revisiting them due to a continuation, but then we
                // are not able to define recursive closures.
                //
                // Meanwhile, having moved the bindings out to here, we can easily self-reference,
                // but we are also sharing a reference incorrectly across continuations that have
                // been left from.
                //
                // We just have to explicitly clear these variables when re-entering a continuation
                // in which the variables are declared but not initialized.
                //
                // However, variables that actually ARE from the scope do need to be shared. It's
                // only THESE variables if the continuation is from within the expression.
                for id in Bindings::of(&unif.pattern) {
                    self.variable(&id);
                }
                let value = self.compile_expression(&unif.expression, "let.expr")?;
                let on_fail = self.get_end("let.fail");
                for id in Bindings::of(&unif.pattern) {
                    let var = self.get_variable(&id).unwrap().ptr();
                    self.trilogy_value_destroy(var);
                }
                self.compile_pattern_match(&unif.pattern, value, on_fail)?;
                if let Some(temp) = self.use_owned_temporary(value) {
                    self.trilogy_value_destroy(temp);
                }
                if let Some(temp) = self.use_owned_temporary(on_fail) {
                    self.trilogy_value_destroy(temp);
                }
                self.compile_expression(&decl.body, name)
            }
            _ => todo!("non-deterministic branching {:?}", decl.query.value),
        }
    }

    fn compile_match(&self, expr: &ir::Match, name: &str) -> Option<PointerValue<'ctx>> {
        let discriminant = self.compile_expression(&expr.expression, "match.discriminant")?;
        self.bind_temporary(discriminant);

        let continuation = self.add_continuation("");
        let mut merger = Merger::default();
        let mut returns = false;
        for case in &expr.cases {
            let next_case_function = self.add_continuation("match.next");
            let (go_to_next_case, next_case_cp) =
                self.capture_current_continuation(next_case_function, "match.next");
            let discriminant = self.use_temporary(discriminant).unwrap();
            self.compile_pattern_match(&case.pattern, discriminant, go_to_next_case)?;
            let Some(guard_bool) = self.compile_expression(&case.guard, "match.guard") else {
                self.become_continuation_point(next_case_cp);
                self.begin_next_function(next_case_function);
                continue;
            };
            let guard_flag = self.trilogy_boolean_untag(guard_bool, "");
            // NOTE: bool doesn't really need to be destroyed... but do it anyway
            self.trilogy_value_destroy(guard_bool);
            let next_block = self.context.append_basic_block(self.get_function(), "next");
            let body_block = self.context.append_basic_block(self.get_function(), "body");
            let body_cp = self.branch_continuation_point();
            self.builder
                .build_conditional_branch(guard_flag, body_block, next_block)
                .unwrap();

            self.builder.position_at_end(next_block);
            let go_next = self.use_temporary(go_to_next_case).unwrap();
            self.void_call_continuation(go_next);

            self.builder.position_at_end(body_block);
            self.become_continuation_point(body_cp);
            if let Some(result) = self.compile_expression(&case.body, name) {
                let closure_allocation = self.continue_in_scope(continuation, result);
                self.end_continuation_point_as_merge(&mut merger, closure_allocation);
                returns = true;
            }

            self.become_continuation_point(next_case_cp);
            self.begin_next_function(next_case_function);
        }

        self.builder.build_unreachable().unwrap();

        if returns {
            if let Some(discriminant_owned) = self.use_owned_temporary(discriminant) {
                self.trilogy_value_destroy(discriminant_owned);
            }
            self.merge_without_branch(merger);
            self.begin_next_function(continuation);
            Some(self.get_continuation(name))
        } else {
            None
        }
    }

    fn compile_application(
        &self,
        application: &ir::Application,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        match &application.function.value {
            Value::Builtin(Builtin::Pin) => {
                return self.compile_expression(&application.argument, name);
            }
            Value::Builtin(builtin) if builtin.is_unary() => {
                return self.compile_apply_unary(*builtin, &application.argument, name);
            }
            Value::Application(app) => match &app.function.value {
                Value::Builtin(builtin) if builtin.is_binary() => {
                    return self.compile_apply_binary(
                        *builtin,
                        &app.argument,
                        &application.argument,
                        name,
                    );
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
                let function = self.use_temporary(function).unwrap();
                for arg in arguments.iter_mut() {
                    *arg = self.use_temporary(*arg).unwrap();
                }
                Some(self.call_procedure(function, &arguments, name))
            }
            // Function application
            _ => {
                let argument = self.compile_expression(&application.argument, "")?;
                let function = self.use_temporary(function).unwrap();
                Some(self.apply_function(function, argument, name))
            }
        }
    }

    fn compile_module_access(
        &self,
        module_ref: &ir::Expression,
        ident: &syntax::Identifier,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        // Possibly a static module reference, which we can support very easily and efficiently
        if let Value::Reference(module) = &module_ref.value
            && let Some(module) =
                self.globals
                    .get(&module.id)
                    .and_then(|global| match &global.head {
                        Head::Module => {
                            let prefix = global
                                .path
                                .iter()
                                .fold(self.location.to_owned(), |p, s| format!("{p}::{s}"));
                            Some(format!("{prefix}::{}", module.id))
                        }
                        Head::ExternalModule(path) => Some(path.to_owned()),
                        _ => None,
                    })
        {
            let target = self.allocate_value(name);
            let declared = self
                .module
                .get_function(&format!("{module}::{}", ident.as_ref()))
                .unwrap();
            self.call_internal(target, declared, &[]);
            return Some(target);
        }

        let module_value = self.compile_expression(module_ref, "")?;
        let module = self.trilogy_module_untag(module_value, "");
        let id = self.atom_value_raw(ident.as_ref().to_owned());
        let target = self.allocate_value(name);
        self.trilogy_module_find(target, module, self.context.i64_type().const_int(id, false));
        self.trilogy_value_destroy(module_value);
        Some(target)
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
            Value::Application(app) => match &app.function.value {
                Value::Application(parent)
                    if matches!(parent.function.value, Value::Builtin(Builtin::Access)) =>
                {
                    let container = self.compile_expression(&parent.argument, "")?;
                    self.bind_temporary(container);
                    let key = self.compile_expression(&app.argument, "")?;
                    self.bind_temporary(key);
                    let value = self.compile_expression(&assign.rhs, "")?;
                    let container = self.use_temporary(container).unwrap();
                    let key = self.use_temporary(key).unwrap();
                    let out = self.allocate_value(name);
                    self.member_assign(out, container, key, value);
                    Some(out)
                }
                _ => panic!("invalid lvalue in assignment"),
            },
            _ => panic!("invalid lvalue in assignment"),
        }
    }

    fn compile_reference(&self, identifier: &ir::Identifier, name: &str) -> PointerValue<'ctx> {
        match self.get_variable(&identifier.id) {
            Some(Variable::Owned(variable)) => {
                let target = self.allocate_value(name);
                self.trilogy_value_clone_into(target, variable);
                target
            }
            Some(Variable::Closed { location, .. }) => {
                let target = self.allocate_value(name);
                self.trilogy_value_clone_into(target, location);
                target
            }
            None => {
                let global = self
                    .globals
                    .get(&identifier.id)
                    .expect("unresolved variable");
                self.reference_global(global, name)
            }
        }
    }

    fn reference_global(&self, global: &Global, name: &str) -> PointerValue<'ctx> {
        let target = self.allocate_value(name);
        let global_name = format!("{}::{}", global.module_path(&self.location), global.id);
        let function = self
            .module
            .get_function(&global_name)
            .expect("function was not defined");
        if function.count_params() == 2 {
            self.call_internal(target, function, &[self.get_closure("").into()]);
        } else {
            self.call_internal(target, function, &[]);
        }
        target
    }

    fn compile_or(
        &self,
        lhs: &ir::Expression,
        rhs: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        // Compile the condition first, in case it branches
        let lhs_value = self.compile_expression(lhs, "or.lhs")?;

        // Then save the current context: this is the place from which we are branching.
        let original_function_scope = self.get_function();
        let snapshot = self.snapshot_function_context();
        let if_true_block = self
            .context
            .append_basic_block(original_function_scope, "or.true");
        let if_false_block = self
            .context
            .append_basic_block(original_function_scope, "or.false");

        let if_false_function = self.add_continuation("or.false");
        let merge_to_function = self.add_continuation("or.cont");
        let mut merger = Merger::default();

        let cond_bool = self.trilogy_boolean_untag(lhs_value, "");
        self.builder
            .build_conditional_branch(cond_bool, if_true_block, if_false_block)
            .unwrap();
        let false_cp = self.branch_continuation_point();

        self.builder.position_at_end(if_true_block);
        let if_true_closure = self.continue_in_scope(merge_to_function, lhs_value);
        self.end_continuation_point_as_merge(&mut merger, if_true_closure);

        self.become_continuation_point(false_cp);
        self.builder.position_at_end(if_false_block);
        self.restore_function_context(snapshot);
        let if_false_closure = self.void_continue_in_scope(if_false_function);
        self.end_continuation_point_as_close(if_false_closure);

        self.begin_next_function(if_false_function);
        let when_false = self.compile_expression(rhs, name);

        if let Some(value) = when_false {
            let continue_in_scope = self.continue_in_scope(merge_to_function, value);
            self.end_continuation_point_as_merge(&mut merger, continue_in_scope);
        }

        self.merge_without_branch(merger);
        self.begin_next_function(merge_to_function);
        Some(self.get_continuation(name))
    }

    fn compile_and(
        &self,
        lhs: &ir::Expression,
        rhs: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        // Compile the condition first, in case it branches
        let lhs_value = self.compile_expression(lhs, "and.lhs")?;

        // Then save the current context: this is the place from which we are branching.
        let original_function_scope = self.get_function();
        let snapshot = self.snapshot_function_context();
        let if_true_block = self
            .context
            .append_basic_block(original_function_scope, "and.true");
        let if_false_block = self
            .context
            .append_basic_block(original_function_scope, "and.false");

        let if_true_function = self.add_continuation("and.true");
        let merge_to_function = self.add_continuation("and.cont");
        let mut merger = Merger::default();

        let cond_bool = self.trilogy_boolean_untag(lhs_value, "");
        self.builder
            .build_conditional_branch(cond_bool, if_true_block, if_false_block)
            .unwrap();
        let true_cp = self.branch_continuation_point();

        self.builder.position_at_end(if_false_block);
        let if_false_closure = self.continue_in_scope(merge_to_function, lhs_value);
        self.end_continuation_point_as_merge(&mut merger, if_false_closure);

        self.become_continuation_point(true_cp);
        self.builder.position_at_end(if_true_block);
        self.restore_function_context(snapshot);
        let if_true_closure = self.void_continue_in_scope(if_true_function);
        self.end_continuation_point_as_close(if_true_closure);

        self.begin_next_function(if_true_function);
        let when_true = self.compile_expression(rhs, name);

        if let Some(value) = when_true {
            let continue_in_scope = self.continue_in_scope(merge_to_function, value);
            self.end_continuation_point_as_merge(&mut merger, continue_in_scope);
        }

        self.merge_without_branch(merger);
        self.begin_next_function(merge_to_function);
        Some(self.get_continuation(name))
    }

    fn compile_if_else(&self, if_else: &ir::IfElse, name: &str) -> Option<PointerValue<'ctx>> {
        // Compile the condition first, in case it branches
        let condition = self.compile_expression(&if_else.condition, "if.cond")?;

        // Then save the current context: this is the place from which we are branching.
        let original_function_scope = self.get_function();
        let snapshot = self.snapshot_function_context();
        let if_true_block = self
            .context
            .append_basic_block(original_function_scope, "if.true");
        let if_false_block = self
            .context
            .append_basic_block(original_function_scope, "if.false");

        let if_true_function = self.add_continuation("if.true");
        let if_false_function = self.add_continuation("if.false");
        let merge_to_function = self.add_continuation("if.cont");
        let mut merger = Merger::default();

        let cond_bool = self.trilogy_boolean_untag(condition, "if.cond");
        self.trilogy_value_destroy(condition);
        self.builder
            .build_conditional_branch(cond_bool, if_true_block, if_false_block)
            .unwrap();
        let false_cp = self.branch_continuation_point();

        self.builder.position_at_end(if_true_block);
        let if_true_closure = self.void_continue_in_scope(if_true_function);
        self.end_continuation_point_as_close(if_true_closure);

        self.begin_next_function(if_true_function);
        let when_true = self.compile_expression(&if_else.when_true, name);

        if let Some(value) = when_true {
            // Given that the expression eventually evaluates, this branch must merge.
            let after_true_closure = self.continue_in_scope(merge_to_function, value);
            self.end_continuation_point_as_merge(&mut merger, after_true_closure);
        }

        self.become_continuation_point(false_cp);
        self.builder.position_at_end(if_false_block);
        self.restore_function_context(snapshot);
        let if_false_closure = self.void_continue_in_scope(if_false_function);
        self.end_continuation_point_as_close(if_false_closure);

        self.begin_next_function(if_false_function);
        let when_false = self.compile_expression(&if_else.when_false, name);

        if let Some(value) = when_false {
            let continue_in_scope = self.continue_in_scope(merge_to_function, value);
            self.end_continuation_point_as_merge(&mut merger, continue_in_scope);
        }

        if when_true.is_some() || when_false.is_some() {
            self.merge_without_branch(merger);
            self.begin_next_function(merge_to_function);
            Some(self.get_continuation(name))
        } else {
            None
        }
    }

    fn compile_do(&self, procedure: &ir::Procedure, name: &str) -> PointerValue<'ctx> {
        let (current, _, _) = self.get_current_definition();
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

        self.trilogy_callable_init_do(target, arity, closure, function);
        let here = self.builder.get_insert_block().unwrap();
        let snapshot = self.snapshot_function_context();

        let outer_cp = self.shadow_continuation_point();
        let inner_cp = self.capture_contination_point(closure.as_instruction_value().unwrap());
        self.become_continuation_point(inner_cp);
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
        self.restore_function_context(snapshot);
        self.become_continuation_point(outer_cp);
        target
    }

    fn compile_fn(&self, func: &ir::Function, name: &str) -> PointerValue<'ctx> {
        let (current, _, _) = self.get_current_definition();
        let function_name = format!("{current}<fn@{}>", func.span);
        let arity = func.parameters.len();
        let function = self.module.add_function(
            &function_name,
            self.procedure_type(1, true),
            Some(Linkage::Internal),
        );

        let target = self.allocate_value(name);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        self.trilogy_callable_init_fn(target, closure, function);
        let here = self.builder.get_insert_block().unwrap();
        let snapshot = self.snapshot_function_context();

        let outer_cp = self.shadow_continuation_point();
        let inner_cp = self.capture_contination_point(closure.as_instruction_value().unwrap());
        self.become_continuation_point(inner_cp);
        let procedure_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            &function_name,
            None,
            self.di.unit.get_file(),
            func.span.start().line as u32 + 1,
            self.di.closure_di_type(arity),
            true,
            true,
            func.span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        function.set_subprogram(procedure_scope);
        self.compile_function_body(function, &[func], func.span);

        self.builder.position_at_end(here);
        self.restore_function_context(snapshot);
        self.become_continuation_point(outer_cp);
        target
    }

    fn compile_qy(&self, rule: &ir::Rule, name: &str) -> PointerValue<'ctx> {
        let (current, _, _) = self.get_current_definition();
        let rule_name = format!("{current}<qy@{}>", rule.span);
        let arity = rule.parameters.len();
        let function = self.module.add_function(
            &rule_name,
            self.procedure_type(arity, true),
            Some(Linkage::Internal),
        );

        let target = self.allocate_value(name);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        self.trilogy_callable_init_qy(target, arity, closure, function);
        let here = self.builder.get_insert_block().unwrap();
        let snapshot = self.snapshot_function_context();

        let outer_cp = self.shadow_continuation_point();
        let inner_cp = self.capture_contination_point(closure.as_instruction_value().unwrap());
        self.become_continuation_point(inner_cp);
        let procedure_scope = self.di.builder.create_function(
            self.di.unit.get_file().as_debug_info_scope(),
            &rule_name,
            None,
            self.di.unit.get_file(),
            rule.span.start().line as u32 + 1,
            self.di.closure_di_type(arity),
            true,
            true,
            rule.span.start().line as u32 + 1,
            LLVMDIFlagPublic,
            false,
        );
        function.set_subprogram(procedure_scope);
        self.compile_rule_body(function, &[rule], rule.span);

        self.builder.position_at_end(here);
        self.restore_function_context(snapshot);
        self.become_continuation_point(outer_cp);
        target
    }
}
