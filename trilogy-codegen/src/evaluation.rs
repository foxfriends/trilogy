use crate::prelude::*;
use std::cell::Cell;
use std::collections::HashSet;
use trilogy_ir::ir;
use trilogy_ir::visitor::{HasBindings, HasCanEvaluate, IrVisitable, IrVisitor};
use trilogy_vm::{Annotation, Array, Instruction, Location, Offset, Record, Set};

struct Evaluator<'b, 'a> {
    context: &'b mut Context<'a>,
}

pub(crate) trait CodegenEvaluate: IrVisitable {
    fn evaluate(&self, context: &mut Context) {
        self.visit(&mut Evaluator { context });
    }
}

impl CodegenEvaluate for ir::Expression {
    fn evaluate(&self, context: &mut Context) {
        let start = context.ip();
        self.visit(&mut Evaluator { context });
        let end = context.ip();
        context.annotate(Annotation::source(
            start,
            end,
            "<intermediate>".to_owned(),
            Location::new(context.location(), self.span),
        ));
    }
}

impl CodegenEvaluate for ir::Value {}

impl IrVisitor for Evaluator<'_, '_> {
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
        self.context.r#while(&node.condition, &node.body);
    }

    fn visit_assignment(&mut self, assignment: &ir::Assignment) {
        match &assignment.lhs.value {
            ir::Value::Reference(var) => {
                self.context.evaluate(&assignment.rhs);
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
                    self.context.reference(ASSIGN).intermediate();
                    self.context.evaluate(collection).intermediate();
                    self.context.evaluate(key).intermediate();
                    self.context
                        .evaluate(&assignment.rhs)
                        .end_intermediate()
                        .end_intermediate()
                        .end_intermediate()
                        .call_procedure(3);
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
        self.context.instruction(Instruction::Unit);
    }

    fn visit_atom(&mut self, value: &str) {
        self.context.atom(value);
    }

    fn visit_iterator(&mut self, iter: &ir::Iterator) {
        self.context
            .iterator(iter, None, None)
            .instruction(Instruction::Unit);
    }

    fn visit_handled(&mut self, handled: &ir::Handled) {
        self.context.with(
            |context, params| {
                for handler in &handled.handlers {
                    context
                        .case(|context, next| {
                            context.declare_variables(handler.pattern.bindings());
                            context
                                .instruction(Instruction::LoadLocal(params.effect))
                                .pattern_match(&handler.pattern, next)
                                .evaluate(&handler.guard)
                                .typecheck("boolean")
                                .cond_jump(next)
                                .push_cancel(params.cancel)
                                .push_resume(params.resume)
                                .evaluate(&handler.body)
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
                context.evaluate(&handled.expression);
            },
        );
    }

    fn visit_let(&mut self, decl: &ir::Let) {
        if decl.query.is_once() {
            let reenter = self.context.make_label("let");
            let declared = self.context.declare_variables(decl.query.bindings());
            self.context
                .prepare_query(&decl.query)
                .label(reenter.clone())
                .execute_query(&decl.query, END)
                // After running the query, we don't need the state anymore
                .instruction(Instruction::Pop)
                .evaluate(&decl.body)
                .instruction(Instruction::Slide(declared as Offset))
                // After the body has been executed, the variables bound in the query are dropped
                .undeclare_variables(decl.query.bindings(), true);
        } else {
            let reenter = self.context.make_label("let");
            let declared = self.context.declare_variables(decl.query.bindings());
            self.context
                .prepare_query(&decl.query)
                .label(reenter.clone())
                .execute_query(&decl.query, END)
                .intermediate(); // query state
            self.context
                .instruction(Instruction::True)
                .instruction(Instruction::False)
                .instruction(Instruction::Branch)
                .cond_jump(&reenter)
                .instruction(Instruction::Pop)
                .end_intermediate()
                .evaluate(&decl.body)
                .instruction(Instruction::Slide(declared as Offset))
                .undeclare_variables(decl.query.bindings(), true);
        }
    }

    fn visit_if_else(&mut self, cond: &ir::IfElse) {
        let when_false = self.context.make_label("else");
        let end = self.context.make_label("end_if");
        self.context
            .evaluate(&cond.condition)
            .typecheck("boolean")
            .cond_jump(&when_false)
            .evaluate(&cond.when_true)
            .jump(&end)
            .label(when_false)
            .evaluate(&cond.when_false)
            .label(end);
    }

    fn visit_match(&mut self, cond: &ir::Match) {
        let val = self.context.evaluate(&cond.expression).intermediate();
        let end = self.context.make_label("match_end");
        for case in &cond.cases {
            let cleanup = self.context.make_label("case_cleanup");
            let vars = self.context.declare_variables(case.pattern.bindings());
            self.context
                .instruction(Instruction::LoadLocal(val))
                .pattern_match(&case.pattern, &cleanup)
                .evaluate(&case.guard)
                .typecheck("boolean")
                .cond_jump(&cleanup)
                .evaluate(&case.body)
                .instruction(Instruction::SetLocal(val))
                .undeclare_variables(case.pattern.bindings(), true)
                .jump(&end)
                .label(cleanup);
            for _ in 0..vars {
                self.context.instruction(Instruction::Pop);
            }
        }
        self.context.end_intermediate().label(end);
    }

    fn visit_fn(&mut self, closure: &ir::Function) {
        let arity = closure.parameters.len();
        let start = self.context.ip();
        self.context.fn_closure(arity, |context| {
            let params = context.scope.closure(arity);
            for (i, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context
                    .instruction(Instruction::LoadLocal(params + i as Offset))
                    .pattern_match(parameter, END);
            }
            context.evaluate(&closure.body);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
            }
            context.scope.unclosure(arity);
        });
        let end = self.context.ip();
        self.context.annotate(Annotation::source(
            start,
            end,
            "<anonymous function>".to_owned(),
            Location::new(self.context.location(), closure.span),
        ));
    }

    fn visit_do(&mut self, closure: &ir::Procedure) {
        let arity = closure.parameters.len();
        let start = self.context.ip();
        self.context.do_closure(arity, |context| {
            let param_start = context.scope.closure(arity);
            for (offset, parameter) in closure.parameters.iter().enumerate() {
                context.declare_variables(parameter.bindings());
                context
                    .instruction(Instruction::LoadLocal(param_start + offset as Offset))
                    .pattern_match(parameter, END);
            }
            context
                .evaluate(&closure.body)
                .instruction(Instruction::Unit)
                .instruction(Instruction::Return);
            for parameter in closure.parameters.iter().rev() {
                context.undeclare_variables(parameter.bindings(), false);
            }
            context.scope.unclosure(arity);
        });
        let end = self.context.ip();
        self.context.annotate(Annotation::source(
            start,
            end,
            "<anonymous procedure>".to_owned(),
            Location::new(self.context.location(), closure.span),
        ));
    }

    fn visit_qy(&mut self, closure: &ir::Rule) {
        let arity = closure.parameters.len();
        let param_start = Cell::new(0);
        let start = self.context.ip();
        self.context.qy_closure(
            arity,
            |context| {
                param_start.set(context.scope.closure(arity));
                // We have to build a bindset, and then extend the state of the query from that,
                // much like in the definition of a rule.
                context
                    .constant(HashSet::new())
                    .instruction(Instruction::SetRegister(TEMPORARY));
                for (offset, parameter) in closure.parameters.iter().enumerate() {
                    let skip = context.make_label("skip");
                    context.declare_variables(parameter.bindings());
                    context
                        .instruction(Instruction::IsSetLocal(
                            param_start.get() + offset as Offset,
                        ))
                        .cond_jump(&skip);
                    // This parameter was set, so fill its bindings and mark them down in the bindset.
                    context.instruction(Instruction::LoadRegister(TEMPORARY));
                    for var in parameter.bindings() {
                        let index = context.scope.lookup(&var).unwrap().unwrap_local();
                        context.constant(index).instruction(Instruction::Insert);
                    }
                    context.intermediate(); // bindset
                    context
                        .instruction(Instruction::LoadLocal(param_start.get() + offset as Offset))
                        .pattern_match(parameter, END)
                        .end_intermediate() // bindset
                        .instruction(Instruction::SetRegister(TEMPORARY))
                        .label(skip);
                }
                context
                    .instruction(Instruction::LoadRegister(TEMPORARY))
                    .extend_query_state(&closure.body);
            },
            |context, state| {
                let on_fail = context.make_label("qy_fail");
                context
                    .instruction(Instruction::LoadLocal(state))
                    .execute_query(&closure.body, &on_fail)
                    .instruction(Instruction::SetLocal(state));

                // Stack these up in reverse so that when the caller starts pattern matching they are
                // doing it left to right, as expected.
                for (i, param) in closure.parameters.iter().enumerate().rev() {
                    let eval = context.make_label("eval");
                    let next = context.make_label("next");
                    context
                        .instruction(Instruction::IsSetLocal(param_start.get() + i as Offset))
                        // Previously unset parameters get evaluated into
                        .cond_jump(&eval)
                        // Previously set parameters are just loaded back up directly
                        .instruction(Instruction::LoadLocal(param_start.get() + i as Offset))
                        .jump(&next);
                    context.label(eval);
                    if param.can_evaluate() {
                        context.evaluate(param);
                    } else {
                        context.instruction(Instruction::Fizzle);
                    }
                    context.scope.intermediate(); // As is each subsequent parameter value
                    context.label(next);
                }

                // The return value is a (backwards) list
                context.instruction(Instruction::Unit);
                for _ in &closure.parameters {
                    context
                        .instruction(Instruction::Swap)
                        .instruction(Instruction::Cons);
                    context.scope.end_intermediate();
                }
                // Finally, put the return value into 'next()
                context
                    .atom("next")
                    .instruction(Instruction::Construct)
                    .instruction(Instruction::Return);
                context
                    .label(on_fail)
                    .atom("done")
                    .instruction(Instruction::Return);
                for parameter in closure.parameters.iter().rev() {
                    context.undeclare_variables(parameter.bindings(), false);
                }
                context.scope.unclosure(arity);
            },
        );
        let end = self.context.ip();
        self.context.annotate(Annotation::source(
            start,
            end,
            "<anonymous query>".to_owned(),
            Location::new(self.context.location(), closure.span),
        ));
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

    fn visit_assert(&mut self, assert: &ir::Assert) {
        let failed = self.context.make_label("assertion_error");
        self.context
            .evaluate(&assert.assertion)
            .instruction(Instruction::Copy)
            .typecheck("boolean")
            .cond_jump(&failed)
            .bubble(|context| {
                context
                    .label(failed)
                    .evaluate(&assert.message)
                    .atom("AssertionError")
                    .instruction(Instruction::Construct)
                    .instruction(Instruction::Panic);
            });
    }

    fn visit_end(&mut self) {
        self.context.instruction(Instruction::Fizzle);
    }

    fn visit_for(&mut self, iter: &ir::Iterator) {
        self.context.r#for(&iter.query, &iter.value);
    }

    fn visit_module_access(
        &mut self,
        (module_ref, ident): &(ir::Expression, trilogy_parser::syntax::Identifier),
    ) {
        self.context
            .evaluate_annotated(module_ref, "<intermediate>", module_ref.span)
            .typecheck("callable")
            .atom(&*ident)
            .call_module();
    }

    fn visit_application(&mut self, application: &ir::Application) {
        match unapply_2(application) {
            (None, ir::Value::Builtin(builtin), arg) if is_unary_operator(*builtin) => {
                self.context
                    .evaluate_annotated(arg, "<intermediate>", application.argument.span);
                write_operator(self.context, *builtin);
            }
            (Some(ir::Value::Builtin(builtin)), lhs, rhs) if is_operator(*builtin) => {
                self.context
                    .evaluate_annotated(
                        lhs,
                        "<intermediate>",
                        application
                            .function
                            .value
                            .as_application()
                            .unwrap()
                            .argument
                            .span,
                    )
                    .intermediate();
                self.context
                    .evaluate_annotated(rhs, "<intermediate>", application.argument.span)
                    .end_intermediate();
                write_operator(self.context, *builtin);
            }
            (None, ir::Value::Builtin(ir::Builtin::Record), ir::Value::Pack(pack)) => {
                self.context.constant(Record::default()).intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        self.context
                            .evaluate(&element.expression)
                            .typecheck("record")
                            .instruction(Instruction::Glue);
                    } else if let ir::Value::Mapping(mapping) = &element.expression.value {
                        self.context.evaluate(&mapping.0).intermediate();
                        self.context
                            .evaluate(&mapping.1)
                            .end_intermediate()
                            .instruction(Instruction::Assign);
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
                    self.context.evaluate(&element.expression);
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
                    self.context.evaluate(&element.expression);
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
            (None, ir::Value::Builtin(ir::Builtin::Is), ir::Value::Query(query)) => {
                let is_fail = self.context.make_label("is_fail");
                let var_count = self.context.declare_variables(query.bindings());
                self.context
                    .prepare_query(&**query)
                    .execute_query(&**query, &is_fail)
                    .constant(true)
                    .bubble(|c| {
                        c.label(&is_fail).constant(false);
                    })
                    .instruction(Instruction::Slide(var_count as Offset + 1));
                self.context.undeclare_variables(query.bindings(), false);
                for _ in 0..=var_count {
                    // One extra POP to discard the query state
                    self.context.instruction(Instruction::Pop);
                }
            }
            _ => {
                self.context
                    .evaluate(&application.function)
                    .typecheck("callable")
                    .intermediate();
                match &application.argument.value {
                    ir::Value::Pack(pack) => {
                        let arity = pack
                            .len()
                            .expect("procedures may not have spread arguments");
                        for element in &pack.values {
                            self.context.evaluate(&element.expression);
                        }
                        self.context.call_procedure(arity);
                    }
                    _ => {
                        self.context.evaluate(&application.argument).call_function();
                    }
                };
                self.context.scope.end_intermediate();
            }
        }
    }
}
