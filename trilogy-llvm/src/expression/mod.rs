use crate::codegen::{ATOM_ASSERTION_FAILED, Codegen, Global, Head, Merger, Variable};
use inkwell::values::{BasicValue, PointerValue};
use trilogy_ir::ir::{self, Builtin, QueryValue, Value};
use trilogy_ir::visitor::{Bindings, HasBindings};
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
            Value::Set(set) => self.compile_set(set, name),
            Value::Record(record) => self.compile_record(record, name),
            Value::ArrayComprehension(comp) => self.compile_array_comprehension(comp, name),
            Value::SetComprehension(comp) => self.compile_set_comprehension(comp, name),
            Value::RecordComprehension(comp) => self.compile_record_comprehension(comp, name),
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
            Value::Pack(..) => panic!("loose packs are not permitted"),
            Value::Mapping(..) => panic!("loose mappings are not permitted"),
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
            self.capture_current_continuation_as_break(break_function, "break");
        let continue_continuation = self.continue_in_loop(continue_function);
        self.push_loop_scope(break_continuation, continue_continuation);
        self.begin_next_function(continue_function);
        // Within the condition, `break` and `cancel` are explicitly not bound, so it doesn't
        // matter which it refers to at this point.
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
        let break_continuation = self.get_break();
        self.call_known_continuation(
            break_continuation,
            self.allocate_const(self.unit_const(), ""),
        );

        self.builder.position_at_end(then_block);
        self.restore_function_context(snapshot);
        self.become_continuation_point(body_cp);
        if let Some(result) = self.compile_expression(&expr.body, name) {
            self.call_known_continuation(self.get_continue(), result);
        }

        self.become_continuation_point(break_continuation_point);
        self.begin_next_function(break_function);
        self.pop_loop_scope();
        Some(self.get_continuation(name))
    }

    fn compile_for(&self, expr: &ir::Iterator, name: &str) -> Option<PointerValue<'ctx>> {
        let done_function = self.add_continuation("done");
        let (done_continuation, done_continuation_point) =
            self.capture_current_continuation_as_break(done_function, "for_break");
        let done_to_clone = self.allocate_value("break");
        self.trilogy_value_clone_into(done_to_clone, done_continuation);
        self.bind_temporary(done_to_clone);
        let next_iteration = self.compile_query_iteration(&expr.query, done_continuation)?;
        self.bind_temporary(next_iteration);
        self.push_loop_scope(done_to_clone, next_iteration);
        if let Some(value) = self.compile_expression(&expr.value, name) {
            let next_iteration = self.use_temporary_clone(next_iteration).unwrap();
            self.trilogy_value_destroy(value);
            self.void_call_continuation(next_iteration);
        }

        self.become_continuation_point(done_continuation_point);
        self.begin_next_function(done_function);
        self.pop_loop_scope();
        // TODO: the for should really somehow be a "fold" construct eventually
        //
        // let [1, 2, 3] = from list = [] for vals(x) { [...list, x] }
        // let [1, 2, 3] = for vals(x) into list = [] { [...list, x] }
        Some(self.allocate_const(self.unit_const(), ""))
    }

    fn compile_array_comprehension(
        &self,
        expr: &ir::Iterator,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        let output = self.allocate_value(name);
        self.trilogy_array_init_cap(output, 8, "");
        self.bind_temporary(output);

        let done_function = self.add_continuation("done");
        let (done_continuation, done_continuation_point) =
            self.capture_current_continuation_as_break(done_function, "for_break");
        let next_iteration = self.compile_query_iteration(&expr.query, done_continuation)?;
        self.bind_temporary(next_iteration);
        if let Some(value) = self.compile_expression(&expr.value, "element") {
            let arr_val = self.use_temporary_clone(output).unwrap();
            let arr = self.trilogy_array_assume(arr_val, "");
            self.trilogy_array_push(arr, value);
            let next_iteration = self.use_temporary_clone(next_iteration).unwrap();
            self.void_call_continuation(next_iteration);
        }

        self.become_continuation_point(done_continuation_point);
        self.begin_next_function(done_function);
        Some(self.use_temporary_clone(output).unwrap())
    }

    fn compile_set_comprehension(
        &self,
        expr: &ir::Iterator,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        let output = self.allocate_value(name);
        self.trilogy_set_init_cap(output, 8, "");
        self.bind_temporary(output);

        let done_function = self.add_continuation("done");
        let (done_continuation, done_continuation_point) =
            self.capture_current_continuation_as_break(done_function, "for_break");
        let next_iteration = self.compile_query_iteration(&expr.query, done_continuation)?;
        self.bind_temporary(next_iteration);
        if let Some(value) = self.compile_expression(&expr.value, "element") {
            let set_val = self.use_temporary_clone(output).unwrap();
            let set = self.trilogy_set_assume(set_val, "");
            self.trilogy_set_insert(set, value);
            let next_iteration = self.use_temporary_clone(next_iteration).unwrap();
            self.void_call_continuation(next_iteration);
        }

        self.become_continuation_point(done_continuation_point);
        self.begin_next_function(done_function);
        Some(self.use_temporary_clone(output).unwrap())
    }

    fn compile_record_comprehension(
        &self,
        expr: &ir::Iterator,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        let output = self.allocate_value(name);
        self.trilogy_record_init_cap(output, 8, "");
        self.bind_temporary(output);

        let done_function = self.add_continuation("done");
        let (done_continuation, done_continuation_point) =
            self.capture_current_continuation_as_break(done_function, "for_break");
        let next_iteration = self.compile_query_iteration(&expr.query, done_continuation)?;
        self.bind_temporary(next_iteration);
        let Value::Mapping(mapping) = &expr.value.value else {
            unreachable!()
        };
        if let Some(key) = self.compile_expression(&mapping.0, "key") {
            self.bind_temporary(key);
            if let Some(value) = self.compile_expression(&mapping.1, "val") {
                let rec_val = self.use_temporary_clone(output).unwrap();
                let rec = self.trilogy_record_assume(rec_val, "");
                self.trilogy_record_insert(rec, self.use_temporary_clone(key).unwrap(), value);
                let next_iteration = self.use_temporary_clone(next_iteration).unwrap();
                self.void_call_continuation(next_iteration);
            }
        }

        self.become_continuation_point(done_continuation_point);
        self.begin_next_function(done_function);
        Some(self.use_temporary_clone(output).unwrap())
    }

    fn compile_handled(&self, handled: &ir::Handled, name: &str) -> Option<PointerValue<'ctx>> {
        let body_function = self.add_continuation("when.handler");
        let handler_function = self.add_yield();

        // Prepare cancel continuation for after the handled section is complete.
        let cancel_to_function = self.add_continuation("when.cancel");
        let (cancel_to, cancel_to_continuation_point) =
            self.capture_current_continuation_as_cancel(cancel_to_function, "cancel");

        // Construct yield continuation that continues into the handler.
        let (handler, handler_continuation_point) =
            self.capture_current_continuation_as_yield(handler_function, "yield");

        // Then enter the handler, given the new yield values.
        let body_closure = self.continue_in_handler(body_function, handler);
        self.push_with_scope(cancel_to);

        self.end_continuation_point_as_close(body_closure);
        self.begin_next_function(body_function);
        if let Some(result) = self.compile_expression(&handled.expression, name) {
            // When the body is evaluated, it will cancel to exit the handled area, returning to
            // the most recent resume if mid-handler, or to the outside when complete.
            let cancel_to = self.use_temporary_clone(cancel_to).unwrap();
            self.call_known_continuation(cancel_to, result);
        }

        // Next compile the handler.
        self.become_continuation_point(handler_continuation_point);
        self.begin_next_function(handler_function);

        self.compile_handlers(&handled.handlers);

        // Then back to the original scope, to continue the evaluation normally outside of the
        // handling construct.
        self.pop_with_scope();
        self.become_continuation_point(cancel_to_continuation_point);
        self.begin_next_function(cancel_to_function);
        Some(self.get_continuation(name))
    }

    fn compile_handlers(&self, handlers: &[ir::Handler]) {
        let resume = self.get_provided_resume();
        let effect = self.get_effect_temporary();

        // The handler works similar to a match expression, but matching against the effect
        // and doing much more control flow work.
        for handler in handlers {
            let next_case_function = self.add_continuation("");
            let (go_to_next_case, next_case_cp) =
                self.capture_current_continuation(next_case_function, "when.next");
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
            let go_next = self.use_temporary_clone(go_to_next_case).unwrap();
            self.void_call_continuation(go_next);

            self.builder.position_at_end(body_block);
            self.destroy_owned_temporary(go_to_next_case);
            self.restore_function_context(snapshot);
            self.become_continuation_point(body_cp);
            self.push_handler_scope(resume);
            if let Some(result) = self.compile_expression(&handler.body, "handler_result") {
                // If a handler runs off, it ends. Most handlers should choose to explicitly cancel
                // at some point.
                self.trilogy_value_destroy(result);
                self.void_call_continuation(self.get_end(""));
            }
            self.pop_handler_scope();

            self.become_continuation_point(next_case_cp);
            self.begin_next_function(next_case_function);
        }

        // A handler is always complete by the syntax requiring an `else` case at the end, so the last
        // branch is never reachable.
        let unreachable = self.builder.build_unreachable().unwrap();
        self.end_continuation_point_as_clean(unreachable);
    }

    fn compile_assertion(&self, assertion: &ir::Assert, name: &str) -> Option<PointerValue<'ctx>> {
        let expression = self.compile_expression(&assertion.assertion, name)?;

        let pass_cp = self.branch_continuation_point();
        let cond = self.trilogy_boolean_untag(expression, "");
        self.trilogy_value_destroy(expression);
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
            let assertion_error = self.allocate_value("");
            self.trilogy_struct_init_new(
                assertion_error,
                self.context
                    .i64_type()
                    .const_int(ATOM_ASSERTION_FAILED, false),
                msg,
            );
            self.call_yield(assertion_error, "");
            let panic_msg = self.allocate_value("");
            self.string_const(panic_msg, "resumed from assertion error\n");
            let panic = self.panic(panic_msg);
            self.builder.build_unreachable().unwrap();
            self.end_continuation_point_as_clean(panic);
        }

        self.builder.position_at_end(pass);
        self.become_continuation_point(pass_cp);
        self.restore_function_context(snapshot);
        Some(self.allocate_const(self.unit_const(), "assertion"))
    }

    fn compile_sequence(&self, seq: &[ir::Expression], name: &str) -> Option<PointerValue<'ctx>> {
        let mut exprs = seq.iter();
        let mut value = self.compile_expression(exprs.next().unwrap(), "")?;
        for expr in exprs {
            self.trilogy_value_destroy(value);
            value = self.compile_expression(expr, name)?;
        }
        Some(value)
    }

    fn compile_array(&self, pack: &ir::Pack, name: &str) -> Option<PointerValue<'ctx>> {
        let temporaries = pack
            .values
            .iter()
            .map(|element| {
                // NOTE[mut-collection-init]: if the execution re-enters this expression, the value
                // must not get added to the same array instance multiple times. Previously this worked
                // by mutating some temporary collection, but we avoid the issue by tracking proper list
                // of values at compile time and constructing the array later.
                let temporary = self.compile_expression(&element.expression, "arr.el")?;
                self.bind_temporary(temporary);
                Some((element.is_spread, temporary))
            })
            .collect::<Option<Vec<_>>>()?;
        let target = self.allocate_value(name);
        let array_value = self.trilogy_array_init_cap(target, pack.values.len(), "arr");
        for (is_spread, temporary) in temporaries {
            let value = self.use_temporary_clone(temporary).unwrap();
            if is_spread {
                self.trilogy_array_append(array_value, value);
            } else {
                self.trilogy_array_push(array_value, value);
            }
        }
        Some(target)
    }

    fn compile_set(&self, pack: &ir::Pack, name: &str) -> Option<PointerValue<'ctx>> {
        let temporaries = pack
            .values
            .iter()
            .map(|element| {
                // NOTE[mut-collection-init]: see above
                let temporary = self.compile_expression(&element.expression, "set.el")?;
                self.bind_temporary(temporary);
                Some((element.is_spread, temporary))
            })
            .collect::<Option<Vec<_>>>()?;
        let target = self.allocate_value(name);
        let set_value = self.trilogy_set_init_cap(target, pack.values.len(), "set");
        for (is_spread, temporary) in temporaries {
            let value = self.use_temporary_clone(temporary).unwrap();
            if is_spread {
                self.trilogy_set_append(set_value, value);
            } else {
                self.trilogy_set_insert(set_value, value);
            }
        }
        Some(target)
    }

    fn compile_record(&self, pack: &ir::Pack, name: &str) -> Option<PointerValue<'ctx>> {
        enum Element<'ctx> {
            KeyValue(PointerValue<'ctx>, PointerValue<'ctx>),
            Spread(PointerValue<'ctx>),
        }
        let temporaries = pack
            .values
            .iter()
            .map(|element| {
                // NOTE[mut-collection-init]: see above
                match &element.expression.value {
                    ir::Value::Mapping(kv) => {
                        let key = self.compile_expression(&kv.0, "rec.k")?;
                        self.bind_temporary(key);
                        let value = self.compile_expression(&kv.1, "rec.v")?;
                        self.bind_temporary(value);
                        Some(Element::KeyValue(key, value))
                    }
                    _ if element.is_spread => {
                        let value = self.compile_expression(&element.expression, "rec.element")?;
                        self.bind_temporary(value);
                        Some(Element::Spread(value))
                    }
                    _ => panic!("record elements must be spread or mapping"),
                }
            })
            .collect::<Option<Vec<_>>>()?;
        let target = self.allocate_value(name);
        let record_value = self.trilogy_record_init_cap(target, pack.values.len(), "record");
        for element in temporaries {
            match element {
                Element::KeyValue(key, value) => {
                    let key = self.use_temporary_clone(key).unwrap();
                    let value = self.use_temporary_clone(value).unwrap();
                    self.trilogy_record_insert(record_value, key, value);
                }
                Element::Spread(value) => {
                    let value = self.use_temporary_clone(value).unwrap();
                    self.trilogy_record_append(record_value, value);
                }
            }
        }
        Some(target)
    }

    fn compile_let(&self, decl: &ir::Let, name: &str) -> Option<PointerValue<'ctx>> {
        match &decl.query.value {
            QueryValue::Direct(unif) if decl.query.is_once() => {
                // NOTE[rec-let]: These variables are tricky... if they are invented only when initialized,
                // they are trivially cleared before revisiting them due to a continuation, but then we
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
                self.bind_temporary(value);
                for id in unif.pattern.bindings() {
                    let var = self.get_variable(&id).unwrap().ptr();
                    self.trilogy_value_destroy(var);
                }
                self.compile_pattern_match(&unif.pattern, value, self.get_end_temporary())?;
                self.destroy_owned_temporary(value);
                self.compile_expression(&decl.body, name)
            }
            _ => {
                let end = self.get_end("");
                self.bind_temporary(end);

                let next_to = self.compile_query_iteration(&decl.query, end)?;
                self.push_execution(next_to);
                self.compile_expression(&decl.body, name)
            }
        }
    }

    fn compile_match(&self, expr: &ir::Match, name: &str) -> Option<PointerValue<'ctx>> {
        let discriminant = self.compile_expression(&expr.expression, "match.discriminant")?;
        self.bind_temporary(discriminant);

        let continuation = self.add_continuation("");
        let mut merger = Merger::default();
        let mut returns = false;
        for case in &expr.cases {
            // An unmatchable case can be skipped; rare this would occur, but easy to handle
            // since we're handling true specially anyway
            if matches!(case.guard.value, Value::Boolean(false)) {
                continue;
            }

            let next_case_function = self.add_continuation("match.next");
            let (go_to_next_case, next_case_cp) =
                self.capture_current_continuation(next_case_function, "match.next");
            self.compile_pattern_match(&case.pattern, discriminant, go_to_next_case)?;
            let Some(guard_bool) = self.compile_expression(&case.guard, "match.guard") else {
                self.become_continuation_point(next_case_cp);
                self.begin_next_function(next_case_function);
                continue;
            };

            if !matches!(case.guard.value, Value::Boolean(true)) {
                let guard_flag = self.trilogy_boolean_untag(guard_bool, "");
                self.trilogy_value_destroy(guard_bool); // NOTE: bool doesn't really need to be destroyed... but do it anyway
                let next_block = self.context.append_basic_block(self.get_function(), "next");
                let body_block = self.context.append_basic_block(self.get_function(), "body");
                let body_cp = self.branch_continuation_point();
                self.builder
                    .build_conditional_branch(guard_flag, body_block, next_block)
                    .unwrap();

                self.builder.position_at_end(next_block);
                let go_next = self.use_temporary_clone(go_to_next_case).unwrap();
                self.void_call_continuation(go_next);

                self.builder.position_at_end(body_block);
                self.become_continuation_point(body_cp);
            }

            self.destroy_owned_temporary(go_to_next_case);
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
            self.destroy_owned_temporary(discriminant);
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
            Value::Builtin(Builtin::Is) => {
                let Value::Query(query) = &application.argument.value else {
                    unreachable!();
                };
                let merge_to_fn = self.add_continuation("is.cont");
                let (merge_to_cont, merge_to_cp) =
                    self.capture_current_continuation(merge_to_fn, "is.cont");

                let end_false_fn = self.add_continuation("is_false");
                let (end_false_cont, end_false_cp) =
                    self.capture_current_continuation(end_false_fn, "is_false");
                let next = self.compile_query_iteration(query, end_false_cont)?;
                self.trilogy_value_destroy(next);
                let result = self.allocate_const(self.bool_const(true), "");
                self.call_known_continuation(
                    self.use_temporary_clone(merge_to_cont).unwrap(),
                    result,
                );

                self.become_continuation_point(end_false_cp);
                self.begin_next_function(end_false_fn);
                let result = self.allocate_const(self.bool_const(false), "");
                self.call_known_continuation(
                    self.use_temporary_clone(merge_to_cont).unwrap(),
                    result,
                );

                self.become_continuation_point(merge_to_cp);
                self.begin_next_function(merge_to_fn);
                return Some(self.get_continuation("is"));
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
        let function = self.compile_expression(&application.function, "fn")?;
        self.bind_temporary(function);
        match &application.argument.value {
            // Procedure application
            Value::Pack(pack) => {
                let mut arguments = Vec::with_capacity(pack.values.len());
                for (i, val) in pack.values.iter().enumerate() {
                    assert!(
                        !val.is_spread,
                        "a spread is not permitted in procedure argument lists"
                    );
                    let param = self.compile_expression(&val.expression, &format!("arg_{i}"))?;
                    self.bind_temporary(param);
                    arguments.push(param);
                }
                let function = self.use_temporary_clone(function).unwrap();
                for arg in arguments.iter_mut() {
                    *arg = self.use_temporary_clone(*arg).unwrap();
                }
                Some(self.call_procedure(function, &arguments, name))
            }
            // Function application
            _ => {
                let argument = self.compile_expression(&application.argument, "arg")?;
                let function = self.use_temporary_clone(function).unwrap();
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
            && let Some(global) = self.globals.get(&module.id)
            && let Some(module) = match &global.head {
                Head::Module => {
                    let prefix = global
                        .path
                        .iter()
                        .fold(self.location.to_owned(), |p, s| format!("{p}::{s}"));
                    Some(format!("{prefix}::{}", module.id))
                }
                Head::ExternalModule(path) => Some(path.to_owned()),
                _ => None,
            }
            && let Some(declared) = self
                .module
                .get_function(&format!("{module}::{}", ident.as_ref()))
        {
            // TODO: statically invalid module accesses could be caught by the compiler
            // and reported, but we'd want to do that before coming through here, as
            // this final compile step is supposed to be without error.
            let target = self.allocate_value(name);
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
                    let container_val = self.use_temporary_clone(container).unwrap();
                    let key_val = self.use_temporary_clone(key).unwrap();
                    let out = self.allocate_value(name);
                    self.member_assign(out, container_val, key_val, value);
                    Some(out)
                }
                _ => panic!("invalid lvalue in assignment"),
            },
            _ => panic!("invalid lvalue in assignment"),
        }
    }

    fn compile_reference(&self, identifier: &ir::Identifier, name: &str) -> PointerValue<'ctx> {
        match self.get_variable(&identifier.id) {
            Some(Variable::Owned(variable)) | Some(Variable::Argument(variable)) => {
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
        let if_false_block = self
            .context
            .append_basic_block(original_function_scope, "and.false");
        let if_true_block = self
            .context
            .append_basic_block(original_function_scope, "and.true");

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
        self.trilogy_value_destroy(lhs_value);
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
        let function =
            self.add_procedure(&function_name, arity, &function_name, procedure.span, true);

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
        self.compile_procedure_body(function, procedure);

        self.builder.position_at_end(here);
        self.restore_function_context(snapshot);
        self.become_continuation_point(outer_cp);
        target
    }

    fn compile_fn(&self, func: &ir::Function, name: &str) -> PointerValue<'ctx> {
        let (current, _, _) = self.get_current_definition();
        let function_name = format!("{current}<fn@{}>", func.span);
        let function = self.add_function(&function_name, &function_name, func.span, true);

        let target = self.allocate_value(name);
        let closure = self
            .builder
            .build_alloca(self.value_type(), "TEMP_CLOSURE")
            .unwrap();

        self.trilogy_callable_init_do(target, 1, closure, function);
        let here = self.builder.get_insert_block().unwrap();
        let snapshot = self.snapshot_function_context();

        let outer_cp = self.shadow_continuation_point();
        let inner_cp = self.capture_contination_point(closure.as_instruction_value().unwrap());
        self.become_continuation_point(inner_cp);
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
        let function = self.add_rule(&rule_name, arity, &rule_name, rule.span, true);

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
        self.compile_rule_body(function, &[rule], rule.span);

        self.builder.position_at_end(here);
        self.restore_function_context(snapshot);
        self.become_continuation_point(outer_cp);
        target
    }
}
