use crate::codegen::{Codegen, Merger};
use crate::types::{TAG_STRUCT, TAG_TUPLE};
use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{IntValue, PointerValue};
use num::{ToPrimitive, Zero};
use trilogy_ir::ir::{self, Builtin, Value};

impl<'ctx> Codegen<'ctx> {
    #[must_use = "must acknowledge continuation of control flow"]
    pub(crate) fn compile_pattern_match(
        &self,
        pattern: &ir::Expression,
        value: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
    ) -> Option<()> {
        let prev = self.set_span(pattern.span);
        self.bind_temporary(value);
        self.bind_temporary(on_fail);

        match &pattern.value {
            Value::Reference(id) => {
                let variable = self.variable(id);
                self.trilogy_value_clone_into(variable, value);
            }
            Value::Conjunction(conj) => {
                self.compile_pattern_match(&conj.0, value, on_fail)?;
                let value = self.use_temporary(value).unwrap().ptr();
                self.compile_pattern_match(&conj.1, value, on_fail)?;
            }
            Value::Disjunction(disj) => {
                // NOTE: somehow... it seems that due to this process we end up corrupting the `return_to` pointer,
                // so that if we exit from the second case and call return, it has already been freed or something,
                // and is full of garbage?
                let on_success_function = self.add_continuation("pm.cont");
                let mut merger = Merger::default();

                let brancher = self.end_continuation_point_as_branch();
                let second_function = self.add_continuation("disj.snd");
                let go_to_second =
                    self.capture_current_continuation(second_function, &brancher, "disj.snd");
                let secondary_cp = self.hold_continuation_point();
                let first_function = self.add_continuation("disj.fst");
                let go_to_first =
                    self.capture_current_continuation(first_function, &brancher, "disj.fst");
                let primary_cp = self.hold_continuation_point();
                self.void_call_continuation(go_to_first, "");
                self.builder.build_unreachable().unwrap();

                self.begin_next_function(first_function);
                self.become_continuation_point(primary_cp);
                let value_ref = self.use_temporary(value).unwrap().ptr();
                self.compile_pattern_match(&disj.0, value_ref, go_to_second)?;
                let closure = self.void_continue_in_scope(on_success_function);
                self.end_continuation_point_as_merge(&mut merger, closure);

                self.begin_next_function(second_function);
                self.become_continuation_point(secondary_cp);
                let value_ref = self.use_temporary(value).unwrap().ptr();
                self.compile_pattern_match(&disj.1, value_ref, on_fail)?;
                let closure = self.void_continue_in_scope(on_success_function);
                self.end_continuation_point_as_merge(&mut merger, closure);

                self.merge_without_branch(merger);
                self.begin_next_function(on_success_function);
            }
            Value::Unit => {
                let constant = self.allocate_const(self.unit_const(), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match, on_fail);
            }
            Value::Boolean(val) => {
                let constant = self.allocate_const(self.bool_const(*val), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match, on_fail);
            }
            Value::Atom(val) => {
                let constant = self.allocate_const(self.atom_const(val.to_owned()), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match, on_fail);
            }
            Value::Character(val) => {
                let constant = self.allocate_const(self.char_const(*val), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match, on_fail);
            }
            Value::Number(num) if num.value().im.is_zero() && num.value().re.is_integer() => {
                if let Some(int) = num.value().re.to_i64() {
                    let constant = self.allocate_const(self.int_const(int), "");
                    let is_match = self.trilogy_value_structural_eq(value, constant, "");
                    self.pm_cont_if(is_match, on_fail);
                } else {
                    todo!("Support non-integers and large integers")
                }
            }
            Value::Bits(bits) => {
                let constant = self.allocate_value("");
                self.bits_const(constant, bits);
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.trilogy_value_destroy(constant);
                self.pm_cont_if(is_match, on_fail);
            }
            Value::String(string) => {
                let constant = self.allocate_value("");
                self.string_const(constant, string);
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.trilogy_value_destroy(constant);
                self.pm_cont_if(is_match, on_fail);
            }
            Value::Application(app) => self.compile_match_application(value, app, on_fail)?,
            Value::Wildcard => {}
            _ => todo!(),
        }

        if let Some(prev) = prev {
            self.overwrite_debug_location(prev);
        }

        Some(())
    }

    fn pm_cont_if(&self, cond: IntValue<'ctx>, on_fail: PointerValue<'ctx>) -> BasicBlock<'ctx> {
        let fail = self
            .context
            .append_basic_block(self.get_function(), "pm_fail");
        let cont = self
            .context
            .append_basic_block(self.get_function(), "pm_cont");

        let brancher = self.end_continuation_point_as_branch();
        self.builder
            .build_conditional_branch(cond, cont, fail)
            .unwrap();
        self.builder.position_at_end(fail);
        let on_fail = self.use_temporary(on_fail).unwrap().ptr();
        self.void_call_continuation(on_fail, "");
        self.builder.build_unreachable().unwrap();

        self.builder.position_at_end(cont);
        self.transfer_debug_info();
        self.resume_continuation_point(&brancher);
        cont
    }

    fn compile_match_application(
        &self,
        value: PointerValue<'ctx>,
        application: &ir::Application,
        on_fail: PointerValue<'ctx>,
    ) -> Option<()> {
        match &application.function.value {
            Value::Builtin(builtin) => {
                self.compile_match_apply_builtin(value, *builtin, &application.argument, on_fail)
            }
            Value::Application(app) => match &app.function.value {
                Value::Builtin(Builtin::Cons) => {
                    let tag = self.get_tag(value, "");
                    let is_tuple = self
                        .builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            tag,
                            self.tag_type().const_int(TAG_TUPLE, false),
                            "",
                        )
                        .unwrap();
                    self.pm_cont_if(is_tuple, on_fail);

                    let tuple = self.trilogy_tuple_assume(value, "");
                    self.bind_temporary(tuple);
                    let left = self.allocate_value("");
                    self.bind_temporary(left);
                    self.trilogy_tuple_left(left, tuple);
                    self.compile_pattern_match(&app.argument, left, on_fail)?;

                    let left = self.use_temporary(left).unwrap().ptr();
                    let tuple = self.use_temporary(tuple).unwrap().ptr();
                    self.trilogy_value_destroy(left);

                    let right = self.allocate_value("");
                    self.bind_temporary(right);
                    self.trilogy_tuple_right(right, tuple);
                    self.compile_pattern_match(&application.argument, right, on_fail)?;
                    let right = self.use_temporary(right).unwrap().ptr();
                    self.trilogy_value_destroy(right);
                    Some(())
                }
                Value::Builtin(Builtin::Construct) => {
                    let tag = self.get_tag(value, "");
                    let is_struct = self
                        .builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            tag,
                            self.tag_type().const_int(TAG_STRUCT, false),
                            "",
                        )
                        .unwrap();
                    self.pm_cont_if(is_struct, on_fail);
                    let destructed = self.allocate_value("");
                    self.destruct(destructed, value);
                    let tuple = self.trilogy_tuple_assume(destructed, "");
                    let part = self.allocate_value("");
                    self.trilogy_tuple_right(part, tuple);
                    self.compile_pattern_match(&app.argument, part, on_fail)?;
                    self.trilogy_value_destroy(part);
                    self.trilogy_tuple_left(part, tuple);
                    self.compile_pattern_match(&application.argument, part, on_fail)?;
                    self.trilogy_value_destroy(part);
                    self.trilogy_value_destroy(destructed);
                    Some(())
                }
                _ => panic!("only builtins can be applied in pattern matching context"),
            },
            _ => panic!("only builtins can be applied in pattern matching context"),
        }
    }

    fn compile_match_apply_builtin(
        &self,
        value: PointerValue<'ctx>,
        builtin: Builtin,
        expression: &ir::Expression,
        on_fail: PointerValue<'ctx>,
    ) -> Option<()> {
        match builtin {
            Builtin::Typeof => {
                // TODO: we should restrict the expressions in this thing to be pins or constants... otherwise we do have to
                // handle branching...
                let expected_type = self.compile_expression(expression, "")?;
                let value = self.use_temporary(value).unwrap().ptr();
                let tag = self.get_tag(value, "");
                let atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                let type_ptr = self.allocate_value("");
                self.trilogy_atom_init(type_ptr, atom);
                let cmp = self.trilogy_value_structural_eq(expected_type, type_ptr, "");
                // NOTE: atom does not require destruction, so type_ptr is ok
                self.pm_cont_if(cmp, on_fail);
            }
            Builtin::Pin => {
                // Because only identifiers can be pinned, we don't have to worry about handling branching mess here
                let pinned = self.compile_expression(expression, "pin")?;
                let cmp = self.trilogy_value_structural_eq(value, pinned, "");
                self.trilogy_value_destroy(pinned);
                self.pm_cont_if(cmp, on_fail);
            }
            _ => todo!(),
        }
        Some(())
    }
}
