use crate::preamble::{END, RETURN};
use crate::{prelude::*, ASSIGN};
use trilogy_ir::ir;
use trilogy_ir::visitor::{HasBindings, IrVisitable, IrVisitor};
use trilogy_parser::syntax;
use trilogy_vm::{Array, Instruction, Record, Set, Value};

#[inline(always)]
pub(crate) fn write_expression(context: &mut Context, expr: &ir::Expression) {
    write_evaluation(context, &expr.value)
}

struct Evaluator<'b, 'a> {
    context: &'b mut Context<'a>,
}

impl IrVisitor for Evaluator<'_, '_> {
    fn visit_query(&mut self, _node: &ir::Query) {
        panic!()
    }

    fn visit_pack(&mut self, _node: &ir::Pack) {
        panic!()
    }

    fn visit_mapping(&mut self, _node: &(ir::Expression, ir::Expression)) {
        panic!()
    }

    fn visit_conjunction(&mut self, _node: &(ir::Expression, ir::Expression)) {
        panic!()
    }

    fn visit_disjunction(&mut self, _node: &(ir::Expression, ir::Expression)) {
        panic!()
    }

    fn visit_wildcard(&mut self) {
        panic!()
    }

    fn visit_builtin(&mut self, builtin: &ir::Builtin) {
        if is_referenceable_operator(*builtin) {
            write_operator_reference(self.context, *builtin);
        } else {
            panic!("{builtin:?} is not a referenceable builtin")
        }
    }

    fn visit_sequence(&mut self, nodes: &[ir::Expression]) {
        self.context.sequence(nodes);
    }

    fn visit_while(&mut self, node: &ir::While) {
        self.context
            .r#while(&node.condition.value, &node.body.value);
    }

    fn visit_assignment(&mut self, assignment: &ir::Assignment) {
        match &assignment.lhs.value {
            ir::Value::Reference(var) => {
                write_expression(self.context, &assignment.rhs);
                match self.context.scope.lookup(&var.id) {
                    Some(Binding::Variable(index)) => {
                        self.context
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::SetLocal(index));
                    }
                    _ => unreachable!("Only variables can be assigned to"),
                }
            }
            ir::Value::Application(application) => match unapply_2(application) {
                (Some(ir::Value::Builtin(ir::Builtin::Access)), collection, key) => {
                    self.context.reference(ASSIGN);
                    write_evaluation(self.context, collection);
                    self.context.scope.intermediate();
                    write_evaluation(self.context, key);
                    self.context.scope.intermediate();
                    write_expression(self.context, &assignment.rhs);
                    self.context.scope.end_intermediate();
                    self.context.scope.end_intermediate();
                    self.context.call_procedure(3);
                }
                _ => unreachable!("LValue applications must be access"),
            },
            _ => unreachable!("LValues must be reference or application"),
        }
    }

    fn visit_number(&mut self, value: &ir::Number) {
        self.context.constant(value.value().clone());
    }

    fn visit_character(&mut self, value: &char) {
        self.context.constant(*value);
    }

    fn visit_string(&mut self, value: &str) {
        self.context.constant(value);
    }

    fn visit_bits(&mut self, value: &ir::Bits) {
        self.context.constant(value.value().clone());
    }

    fn visit_boolean(&mut self, value: &bool) {
        self.context.constant(*value);
    }
    fn visit_unit(&mut self) {
        self.context.constant(());
    }

    fn visit_atom(&mut self, value: &str) {
        self.context.atom(value);
    }

    fn visit_iterator(&mut self, iter: &ir::Iterator) {
        self.context.iterator(iter, None, None).constant(());
    }

    fn visit_handled(&mut self, handled: &ir::Handled) {
        self.context.with(
            |context, params| {
                for handler in &handled.handlers {
                    context
                        .case(|context, next| {
                            context.declare_variables(handler.pattern.bindings());
                            context.instruction(Instruction::LoadLocal(params.effect));
                            write_pattern_match(context, &handler.pattern, next);
                            write_expression(context, &handler.guard);
                            context
                                .typecheck("boolean")
                                .cond_jump(next)
                                .push_cancel(params.cancel)
                                .push_resume(params.resume);
                            write_expression(context, &handler.body);
                            context
                                .pop_cancel()
                                .pop_resume()
                                // At the end of an effect handler, everything just ends.
                                //
                                // The effect handler should have used cancel or resume appropriately
                                // if "just end" was not the intention.
                                .instruction(Instruction::Fizzle);
                        })
                        .undeclare_variables(handler.pattern.bindings(), true);
                }
            },
            |context| {
                write_expression(context, &handled.expression);
            },
        );
    }

    fn visit_let(&mut self, decl: &ir::Let) {
        if decl.query.is_once() {
            let reenter = self.context.make_label("let");
            let declared = self.context.declare_variables(decl.query.bindings());
            write_query_state(self.context, &decl.query);
            self.context.label(reenter.clone());
            write_query(self.context, &decl.query, END);
            // After running the query, we don't need the state anymore
            self.context.instruction(Instruction::Pop);
            write_expression(self.context, &decl.body);
            self.context
                .instruction(Instruction::Slide(declared as u32));
            // After the body has been executed, the variables bound in the query are dropped
            self.context
                .undeclare_variables(decl.query.bindings(), true);
        } else {
            let reenter = self.context.make_label("let");
            let declared = self.context.declare_variables(decl.query.bindings());
            write_query_state(self.context, &decl.query);
            self.context.label(reenter.clone());
            write_query(self.context, &decl.query, END);
            self.context.scope.intermediate();
            self.context
                .instruction(Instruction::Const(Value::Bool(true)))
                .instruction(Instruction::Const(Value::Bool(false)))
                .instruction(Instruction::Branch)
                .cond_jump(&reenter)
                .instruction(Instruction::Pop);
            self.context.scope.end_intermediate();
            write_expression(self.context, &decl.body);
            self.context
                .instruction(Instruction::Slide(declared as u32));
            self.context
                .undeclare_variables(decl.query.bindings(), true);
        }
    }

    fn visit_if_else(&mut self, cond: &ir::IfElse) {
        let when_false = self.context.make_label("else");
        write_expression(self.context, &cond.condition);
        self.context.typecheck("boolean").cond_jump(&when_false);
        write_expression(self.context, &cond.when_true);
        let end = self.context.make_label("end_if");
        self.context.jump(&end);
        self.context.label(when_false);
        write_expression(self.context, &cond.when_false);
        self.context.label(end);
    }

    fn visit_match(&mut self, cond: &ir::Match) {
        write_expression(self.context, &cond.expression);
        let val = self.context.scope.intermediate();
        let end = self.context.make_label("match_end");
        for case in &cond.cases {
            let cleanup = self.context.make_label("case_cleanup");
            let vars = self.context.declare_variables(case.pattern.bindings());
            self.context.instruction(Instruction::LoadLocal(val));
            write_pattern_match(self.context, &case.pattern, &cleanup);
            write_expression(self.context, &case.guard);
            self.context.typecheck("boolean").cond_jump(&cleanup);
            write_expression(self.context, &case.body);
            self.context.instruction(Instruction::SetLocal(val));
            self.context
                .undeclare_variables(case.pattern.bindings(), true);
            self.context.jump(&end);
            self.context.label(cleanup);
            for _ in 0..vars {
                self.context.instruction(Instruction::Pop);
            }
        }
        self.context.scope.end_intermediate();
        self.context.label(end);
    }

    fn visit_fn(&mut self, closure: &ir::Function) {
        let end = self.context.make_label("end_fn");
        let params = self.context.scope.closure(closure.parameters.len());
        for i in 0..closure.parameters.len() {
            self.context
                .close(if i == 0 { &end } else { RETURN })
                .unlock_function();
        }
        for (i, parameter) in closure.parameters.iter().enumerate() {
            self.context.declare_variables(parameter.bindings());
            self.context
                .instruction(Instruction::LoadLocal(params + i as u32));
            write_pattern_match(self.context, parameter, END);
        }
        write_expression(self.context, &closure.body);
        for parameter in closure.parameters.iter().rev() {
            self.context
                .undeclare_variables(parameter.bindings(), false);
            self.context.scope.unclosure(1)
        }

        self.context.label(end);
    }

    fn visit_do(&mut self, closure: &ir::Procedure) {
        let arity = closure.parameters.len();
        let param_start = self.context.scope.closure(arity);
        self.context.proc_closure(arity, |context| {
            for (offset, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context.instruction(Instruction::LoadLocal(param_start + offset as u32));
                write_pattern_match(context, parameter, END);
            }
            write_expression(context, &closure.body);
            context
                .instruction(Instruction::Const(Value::Unit))
                .instruction(Instruction::Return);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
            }
        });
        self.context.scope.unclosure(arity);
    }

    fn visit_reference(&mut self, ident: &ir::Identifier) {
        let binding = self
            .context
            .scope
            .lookup(&ident.id)
            .expect("unresolved reference should not exist at this point");
        match binding {
            Binding::Variable(offset) => {
                self.context.instruction(Instruction::LoadLocal(offset));
            }
            Binding::Static(label) => {
                self.context.reference(label.to_owned());
            }
            Binding::Context(label) => {
                self.context
                    .reference(label.to_owned())
                    .instruction(Instruction::Call(0));
            }
        }
    }

    fn visit_dynamic(&mut self, _value: &syntax::Identifier) {
        panic!();
    }

    fn visit_assert(&mut self, assert: &ir::Assert) {
        let failed = self.context.make_label("assertion_error");
        write_expression(self.context, &assert.assertion);
        self.context
            .instruction(Instruction::Copy)
            .typecheck("boolean")
            .cond_jump(&failed)
            .bubble(|context| {
                context.label(failed);
                write_expression(context, &assert.message);
                context
                    .atom("AssertionError")
                    .instruction(Instruction::Construct)
                    .instruction(Instruction::Panic);
            });
    }

    fn visit_end(&mut self) {
        self.context.instruction(Instruction::Fizzle);
    }

    fn visit_application(&mut self, application: &ir::Application) {
        match unapply_2(application) {
            (
                Some(ir::Value::Builtin(ir::Builtin::ModuleAccess)),
                module_ref,
                ir::Value::Dynamic(ident),
            ) => {
                write_evaluation(self.context, module_ref);
                self.context
                    .typecheck("callable")
                    .atom(&**ident)
                    .call_module();
            }
            (None, ir::Value::Builtin(builtin), arg) if is_unary_operator(*builtin) => {
                write_evaluation(self.context, arg);
                write_operator(self.context, *builtin);
            }
            (Some(ir::Value::Builtin(builtin)), lhs, rhs) if is_operator(*builtin) => {
                write_evaluation(self.context, lhs);
                self.context.scope.intermediate();
                write_evaluation(self.context, rhs);
                self.context.scope.end_intermediate();
                write_operator(self.context, *builtin);
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), ir::Value::Pack(pack)) => {
                self.context.constant(Record::default());
                self.context.scope.intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        write_expression(self.context, &element.expression);
                        self.context
                            .typecheck("record")
                            .instruction(Instruction::Glue);
                    } else if let ir::Value::Mapping(mapping) = &element.expression.value {
                        write_expression(self.context, &mapping.0);
                        self.context.scope.intermediate();
                        write_expression(self.context, &mapping.1);
                        self.context.scope.end_intermediate();
                        self.context.instruction(Instruction::Assign);
                    } else {
                        panic!("record values must be mappings")
                    }
                }
                self.context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), ir::Value::Iterator(iter)) => {
                self.context.comprehension(
                    |context| {
                        context
                            .typecheck("tuple")
                            .constant(Record::new())
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Uncons)
                            .instruction(Instruction::Assign)
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Glue);
                    },
                    |context| {
                        context
                            .iterator(iter, None, None)
                            .constant(Record::default());
                    },
                );
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), ..) => {
                unreachable!("record is applied to pack or iterator");
            }
            (None, ir::Value::Builtin(ir::Builtin::Set), ir::Value::Pack(pack)) => {
                self.context.constant(Set::default());
                self.context.scope.intermediate();
                for element in &pack.values {
                    write_expression(self.context, &element.expression);
                    if element.is_spread {
                        self.context.typecheck("set").instruction(Instruction::Glue);
                    } else {
                        self.context.instruction(Instruction::Insert);
                    }
                }
                self.context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Set), ir::Value::Iterator(iter)) => {
                self.context.comprehension(
                    |context| {
                        context
                            .constant(Set::new())
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Insert)
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Glue);
                    },
                    |context| {
                        context.iterator(iter, None, None).constant(Set::default());
                    },
                );
            }
            (None, ir::Value::Builtin(ir::Builtin::Set), ..) => {
                unreachable!("set is applied to pack or iterator");
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), ir::Value::Pack(pack)) => {
                self.context.constant(Array::default());
                self.context.scope.intermediate();
                for element in &pack.values {
                    self.context.instruction(Instruction::Clone);
                    write_expression(self.context, &element.expression);
                    if element.is_spread {
                        self.context
                            .typecheck("array")
                            .instruction(Instruction::Glue);
                    } else {
                        self.context.instruction(Instruction::Insert);
                    }
                }
                self.context.scope.end_intermediate();
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), ir::Value::Iterator(iter)) => {
                self.context.comprehension(
                    |context| {
                        context
                            .constant(Array::new())
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Insert)
                            .instruction(Instruction::Swap)
                            .instruction(Instruction::Glue);
                    },
                    |context| {
                        context
                            .iterator(iter, None, None)
                            .constant(Array::default());
                    },
                );
            }
            (None, ir::Value::Builtin(ir::Builtin::Array), ..) => {
                unreachable!("array is applied to pack or iterator");
            }
            (None, ir::Value::Builtin(ir::Builtin::For), ir::Value::Iterator(iter)) => {
                self.context.r#for(&iter.query, &iter.value.value);
            }
            (None, ir::Value::Builtin(ir::Builtin::Is), ir::Value::Query(query)) => {
                let is_fail = self.context.make_label("is_fail");
                let var_count = self.context.declare_variables(query.bindings());
                write_query_state(self.context, query);
                write_query(self.context, query, &is_fail);
                self.context
                    .constant(true)
                    .bubble(|c| {
                        c.label(&is_fail).constant(false);
                    })
                    .instruction(Instruction::Slide(var_count as u32 + 1));
                self.context.undeclare_variables(query.bindings(), false);
                for _ in 0..=var_count {
                    // One extra POP to discard the query state
                    self.context.instruction(Instruction::Pop);
                }
            }
            _ => {
                write_expression(self.context, &application.function);
                self.context.typecheck("callable");
                self.context.scope.intermediate();
                match &application.argument.value {
                    ir::Value::Pack(pack) => {
                        let arity = pack
                            .len()
                            .expect("procedures may not have spread arguments");
                        for element in &pack.values {
                            write_expression(self.context, &element.expression);
                        }
                        self.context.call_procedure(arity);
                    }
                    _ => {
                        write_expression(self.context, &application.argument);
                        self.context.call_function();
                    }
                };
                self.context.scope.end_intermediate();
            }
        }
    }
}

pub(crate) fn write_evaluation(context: &mut Context, value: &ir::Value) {
    value.visit(&mut Evaluator { context });
}
