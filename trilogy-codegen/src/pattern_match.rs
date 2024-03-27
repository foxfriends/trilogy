use crate::prelude::*;
use trilogy_ir::ir::{self, Builtin, Expression};
use trilogy_ir::visitor::{IrVisitable, IrVisitor};
use trilogy_vm::{Annotation, Instruction, Location};

struct PatternMatcher<'b, 'a> {
    context: &'b mut Context<'a>,
    on_fail: &'b str,
}

pub(crate) trait CodegenPatternMatch: IrVisitable {
    fn pattern_match(&self, context: &mut Context, on_fail: &str) {
        self.visit(&mut PatternMatcher { context, on_fail });
    }
}

impl CodegenPatternMatch for ir::Value {}
impl CodegenPatternMatch for ir::Expression {
    fn pattern_match(&self, context: &mut Context, on_fail: &str) {
        let start = context.ip();
        self.visit(&mut PatternMatcher { context, on_fail });
        let end = context.ip();
        context.annotate(Annotation::source(
            start,
            end,
            "<pattern>".to_owned(),
            Location::new(context.location(), self.span),
        ));
    }
}

impl IrVisitor for PatternMatcher<'_, '_> {
    fn visit_number(&mut self, value: &ir::Number) {
        self.context
            .constant(value.value().clone())
            .instruction(Instruction::ValEq)
            .cond_jump(self.on_fail);
    }

    fn visit_character(&mut self, value: &char) {
        self.context
            .constant(*value)
            .instruction(Instruction::ValEq)
            .cond_jump(self.on_fail);
    }

    fn visit_string(&mut self, value: &str) {
        self.context
            .constant(value)
            .instruction(Instruction::ValEq)
            .cond_jump(self.on_fail);
    }

    fn visit_bits(&mut self, value: &ir::Bits) {
        self.context
            .constant(value.value().clone())
            .instruction(Instruction::ValEq)
            .cond_jump(self.on_fail);
    }

    fn visit_boolean(&mut self, value: &bool) {
        self.context
            .constant(*value)
            .instruction(Instruction::ValEq)
            .cond_jump(self.on_fail);
    }

    fn visit_unit(&mut self) {
        self.context
            .instruction(Instruction::Unit)
            .instruction(Instruction::ValEq)
            .cond_jump(self.on_fail);
    }

    fn visit_conjunction(&mut self, conj: &(Expression, Expression)) {
        let cleanup = self.context.make_label("conj_cleanup");
        self.context.instruction(Instruction::Copy).intermediate();
        self.context
            .pattern_match(&conj.0, &cleanup)
            .end_intermediate()
            .pattern_match(&conj.1, self.on_fail)
            .bubble(|c| {
                c.label(cleanup)
                    .instruction(Instruction::Pop)
                    .jump(self.on_fail);
            });
    }

    fn visit_disjunction(&mut self, disj: &(Expression, Expression)) {
        let recover = self.context.make_label("disj2");
        self.context.instruction(Instruction::Copy).intermediate();
        self.context
            .pattern_match(&disj.0, &recover)
            .instruction(Instruction::Pop)
            .end_intermediate()
            .bubble(|c| {
                c.label(recover).pattern_match(&disj.1, self.on_fail);
            });
    }

    fn visit_wildcard(&mut self) {
        self.context.instruction(Instruction::Pop);
    }

    fn visit_atom(&mut self, value: &str) {
        self.context
            .atom(value)
            .instruction(Instruction::ValEq)
            .cond_jump(self.on_fail);
    }

    fn visit_reference(&mut self, ident: &ir::Identifier) {
        match self.context.scope.lookup(&ident.id).unwrap() {
            Binding::Variable(offset) => {
                let compare = self.context.make_label("compare");
                self.context
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::InitLocal(offset))
                    .cond_jump(&compare)
                    .instruction(Instruction::Pop)
                    .bubble(|c| {
                        c.label(compare)
                            .instruction(Instruction::LoadLocal(offset))
                            .instruction(Instruction::ValEq)
                            .cond_jump(self.on_fail);
                    });
            }
            Binding::Static(..) | Binding::Context(..) => {
                unreachable!("this is a new binding, so it cannot be static")
            }
        }
    }

    fn visit_application(&mut self, application: &ir::Application) {
        match unapply_2(application) {
            (None, ir::Value::Builtin(Builtin::Negate), value) => {
                let cleanup = self.context.make_label("negate_cleanup");
                self.context
                    .try_type("number", Err(&cleanup))
                    .instruction(Instruction::Negate)
                    .pattern_match(value, self.on_fail)
                    .bubble(|c| {
                        c.label(cleanup)
                            .jump(self.on_fail)
                            .instruction(Instruction::Pop);
                    });
            }
            (None, ir::Value::Builtin(Builtin::Typeof), value) => {
                self.context
                    .instruction(Instruction::TypeOf)
                    .pattern_match(value, self.on_fail);
            }
            (None, ir::Value::Builtin(Builtin::Pin), value) => {
                self.context.evaluate(value);
                self.context
                    .instruction(Instruction::ValEq)
                    .cond_jump(self.on_fail);
            }
            (Some(ir::Value::Builtin(Builtin::Glue)), lhs @ ir::Value::String(..), rhs) => {
                let cleanup = self.context.make_label("glue_cleanup");
                let double_cleanup = self.context.make_label("glue_cleanup2");
                self.context.try_type("string", Err(&cleanup));
                let original = self.context.intermediate();
                let lhs_val = self.context.evaluate(lhs).intermediate();
                self.context
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Take)
                    .instruction(Instruction::LoadLocal(lhs_val))
                    .instruction(Instruction::ValEq)
                    .cond_jump(&double_cleanup)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::Skip)
                    .end_intermediate()
                    .end_intermediate()
                    .pattern_match(rhs, self.on_fail)
                    .bubble(|c| {
                        c.label(double_cleanup)
                            .instruction(Instruction::Pop)
                            .label(cleanup)
                            .instruction(Instruction::Pop)
                            .jump(self.on_fail);
                    });
            }
            (Some(ir::Value::Builtin(Builtin::Glue)), lhs, rhs @ ir::Value::String(..)) => {
                let cleanup = self.context.make_label("glue_cleanup");
                let double_cleanup = self.context.make_label("glue_cleanup2");
                self.context.try_type("string", Err(&cleanup));
                let original = self.context.intermediate();
                let rhs_val = self.context.evaluate(rhs).intermediate();
                self.context
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Length)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Subtract)
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Zero)
                    .instruction(Instruction::Geq)
                    .cond_jump(&double_cleanup)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Skip)
                    .instruction(Instruction::LoadLocal(rhs_val))
                    .instruction(Instruction::ValEq)
                    .cond_jump(&double_cleanup)
                    .instruction(Instruction::Length)
                    .instruction(Instruction::LoadLocal(original))
                    .instruction(Instruction::Length)
                    .instruction(Instruction::Swap)
                    .instruction(Instruction::Subtract)
                    .instruction(Instruction::Take)
                    .end_intermediate()
                    .end_intermediate()
                    .pattern_match(lhs, self.on_fail)
                    .bubble(|c| {
                        c.label(double_cleanup)
                            .instruction(Instruction::Pop)
                            .label(cleanup)
                            .instruction(Instruction::Pop)
                            .jump(self.on_fail);
                    });
            }
            (Some(ir::Value::Builtin(Builtin::Construct)), lhs, rhs) => {
                let cleanup = self.context.make_label("cleanup");
                self.context
                    .try_type("struct", Err(&cleanup))
                    .instruction(Instruction::Destruct)
                    // Match the atom
                    .pattern_match(rhs, &cleanup)
                    // Then match the contents
                    .pattern_match(lhs, self.on_fail)
                    .bubble(|c| {
                        // If the atom matching fails, we have to clean up the extra value
                        c.label(cleanup)
                            .instruction(Instruction::Pop)
                            .jump(self.on_fail);
                    });
            }
            (Some(ir::Value::Builtin(Builtin::Cons)), lhs, rhs) => {
                let cleanup = self.context.make_label("cleanup");
                self.context
                    .try_type("tuple", Err(&cleanup))
                    .instruction(Instruction::Uncons)
                    .instruction(Instruction::Swap)
                    .intermediate(); // rhs
                self.context
                    .pattern_match(lhs, &cleanup)
                    .end_intermediate() // rhs
                    .pattern_match(rhs, self.on_fail)
                    // If the first matching fails, we have to clean up the second
                    .bubble(|c| {
                        c.label(cleanup)
                            .instruction(Instruction::Pop)
                            .jump(self.on_fail);
                    });
            }
            (None, ir::Value::Builtin(Builtin::Array), ir::Value::Pack(pack)) => {
                let cleanup = self.context.make_label("array_cleanup");
                // Before even attempting to match this array, check its length and the length of
                // the pattern. If the pattern is longer than the array, then give up already.
                // The spread element doesn't count towards length since it can be 0. If the pattern
                // is shorter than the array and there is no spread, then also give up.
                let needed = pack
                    .values
                    .iter()
                    .filter(|element| !element.is_spread)
                    .count();
                let cmp = if pack.values.iter().any(|el| el.is_spread) {
                    Instruction::Geq
                } else {
                    Instruction::ValEq
                };
                self.context
                    .try_type("array", Err(&cleanup))
                    .instruction(Instruction::Copy)
                    .instruction(Instruction::Length)
                    .constant(needed)
                    .instruction(cmp)
                    .cond_jump(&cleanup);
                // If that worked, then we'll have enough elements and won't have to check that
                // below at all.

                // Going to be modifying this array in place, so clone it before we begin.
                // Trilogy does not have slices.
                self.context.instruction(Instruction::Clone);
                let array = self.context.scope.intermediate();
                for (i, element) in pack.values.iter().enumerate() {
                    if element.is_spread {
                        // When it's the spread element, take all the elements we aren't going to
                        // need for the tail of this pattern from the array.
                        let cleanup_spread = self.context.make_label("cleanup_spread");
                        let elements_in_tail = pack.values.len() - i - 1;
                        let length = self
                            .context
                            // First determine the runtime length to find out how many elements
                            // we don't need later.
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Length)
                            .constant(elements_in_tail)
                            .instruction(Instruction::Subtract)
                            .intermediate();
                        self.context
                            // Take that many elements from (a copy of) the array
                            .instruction(Instruction::LoadLocal(array))
                            .instruction(Instruction::LoadLocal(length))
                            .instruction(Instruction::Take)
                            // And match that prefix with the element pattern
                            .pattern_match(&element.expression, &cleanup_spread)
                            // Then use the copy of length that's still on the stack to drop those
                            // elements we just took from the original array.
                            .instruction(Instruction::Skip)
                            .bubble(|c| {
                                // If we fail during the spread matching, the length that's on the stack has
                                // to be discarded still.
                                c.label(cleanup_spread)
                                    .instruction(Instruction::Pop)
                                    .jump(&cleanup);
                            })
                            .end_intermediate(); // length
                    } else {
                        // When it's not the spread element, just match the first element.
                        self.context
                            .instruction(Instruction::Copy)
                            .instruction(Instruction::Zero)
                            .instruction(Instruction::Access)
                            .pattern_match(&element.expression, &cleanup)
                            // And then we drop that element from the array and leave just the tail on
                            // the stack.
                            .instruction(Instruction::One)
                            .instruction(Instruction::Skip);
                    }
                }
                // There should now be an empty array on the stack, so get rid of it before continuing.
                self.context
                    .instruction(Instruction::Pop)
                    .bubble(|c| {
                        c.label(cleanup)
                            // Otherwise, we have to cleanup. The only thing on the stack is the array.
                            .instruction(Instruction::Pop)
                            .jump(self.on_fail);
                    })
                    .end_intermediate(); // array
            }
            (None, ir::Value::Builtin(Builtin::Record), ir::Value::Pack(pack)) => {
                let cleanup1 = self.context.make_label("record_cleanup1");
                let cleanup2 = self.context.make_label("record_cleanup2");
                let mut spread = None;
                self.context
                    .try_type("record", Err(&cleanup1))
                    .instruction(Instruction::Clone);
                let record = self.context.scope.intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        spread = Some(&element.expression);
                        continue;
                    }
                    let ir::Value::Mapping(mapping) = &element.expression.value else {
                        panic!("record pattern elements must be mapping ");
                    };
                    let key = self.context.evaluate(&mapping.0).intermediate();
                    self.context
                        .instruction(Instruction::LoadLocal(record))
                        .instruction(Instruction::LoadLocal(key))
                        .instruction(Instruction::Contains)
                        .cond_jump(&cleanup2)
                        .instruction(Instruction::LoadLocal(record))
                        .instruction(Instruction::LoadLocal(key))
                        .instruction(Instruction::Access)
                        .pattern_match(&mapping.1, &cleanup2)
                        .instruction(Instruction::Delete)
                        .end_intermediate();
                }
                self.context.scope.end_intermediate();
                if let Some(spread) = spread {
                    self.context.pattern_match(spread, self.on_fail);
                } else {
                    self.context
                        .instruction(Instruction::Length)
                        .instruction(Instruction::Zero)
                        .instruction(Instruction::ValEq)
                        .cond_jump(self.on_fail);
                }
                self.context.bubble(|c| {
                    c.label(cleanup2)
                        .instruction(Instruction::Pop)
                        .label(cleanup1)
                        .instruction(Instruction::Pop)
                        .jump(self.on_fail);
                });
            }
            (None, ir::Value::Builtin(Builtin::Set), ir::Value::Pack(pack)) => {
                let cleanup1 = self.context.make_label("set_cleanup1");
                let cleanup2 = self.context.make_label("set_cleanup2");
                let mut spread = None;
                self.context
                    .try_type("set", Err(&cleanup1))
                    .instruction(Instruction::Clone);
                let set = self.context.scope.intermediate();
                for element in &pack.values {
                    if element.is_spread {
                        spread = Some(&element.expression);
                        continue;
                    }
                    let value = self.context.evaluate(&element.expression).intermediate();
                    self.context
                        .instruction(Instruction::LoadLocal(set))
                        .instruction(Instruction::LoadLocal(value))
                        .instruction(Instruction::Contains)
                        .cond_jump(&cleanup2)
                        .instruction(Instruction::Delete)
                        .end_intermediate();
                }
                self.context.scope.end_intermediate();
                if let Some(spread) = spread {
                    self.context.pattern_match(spread, self.on_fail);
                } else {
                    self.context
                        .instruction(Instruction::Length)
                        .instruction(Instruction::Zero)
                        .instruction(Instruction::ValEq)
                        .cond_jump(self.on_fail);
                }
                self.context.bubble(|c| {
                    c.label(cleanup2)
                        .instruction(Instruction::Pop)
                        .label(cleanup1)
                        .instruction(Instruction::Pop)
                        .jump(self.on_fail);
                });
            }
            what => panic!("not a pattern ({what:?})"),
        }
    }
}
