use crate::codegen::{Codegen, Merger};
use crate::types::{TAG_STRUCT, TAG_TUPLE};
use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{IntValue, PointerValue};
use trilogy_ir::Id;
use trilogy_ir::ir::{self, Builtin, Value};

impl<'ctx> Codegen<'ctx> {
    #[must_use = "must acknowledge continuation of control flow"]
    pub(crate) fn compile_pattern_match(
        &self,
        pattern: &ir::Expression,
        value: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
    ) -> Option<()> {
        let mut bound_ids = Vec::default();
        self.bind_temporary(value);
        self.bind_temporary(on_fail);
        self.match_pattern(pattern, value, on_fail, &mut bound_ids)
    }

    fn match_pattern(
        &self,
        pattern: &ir::Expression,
        value: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
        bound_ids: &mut Vec<Id>,
    ) -> Option<()> {
        let prev = self.set_span(pattern.span);

        match &pattern.value {
            Value::Reference(id) => {
                bound_ids.push(id.id.clone());
                let variable = self.variable(&id.id);
                let value_ref = self.use_temporary(value).unwrap();
                self.trilogy_value_clone_into(variable, value_ref);
            }
            Value::Conjunction(conj) => {
                self.match_pattern(&conj.0, value, on_fail, bound_ids)?;
                self.match_pattern(&conj.1, value, on_fail, bound_ids)?;
            }
            Value::Disjunction(disj) => {
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
                let bound_before_first_pattern = bound_ids.len();
                self.match_pattern(&disj.0, value, go_to_second, bound_ids)?;
                if let Some(temp) = self.use_owned_temporary(go_to_second) {
                    self.trilogy_value_destroy(temp);
                }
                let closure = self.void_continue_in_scope(on_success_function);
                self.end_continuation_point_as_merge(&mut merger, closure);

                self.begin_next_function(second_function);
                self.become_continuation_point(secondary_cp);
                for id in bound_ids
                    .split_off(bound_before_first_pattern)
                    .into_iter()
                    .filter(|id| !bound_ids.contains(id))
                {
                    let var = self.get_variable(&id).unwrap().ptr();
                    self.trilogy_value_destroy(var);
                }
                self.match_pattern(&disj.1, value, on_fail, bound_ids)?;
                let closure = self.void_continue_in_scope(on_success_function);
                self.end_continuation_point_as_merge(&mut merger, closure);

                self.merge_without_branch(merger);
                self.begin_next_function(on_success_function);
            }
            Value::Unit => {
                let constant = self.allocate_const(self.unit_const(), "");
                self.match_constant(value, constant, on_fail);
            }
            Value::Boolean(val) => {
                let constant = self.allocate_const(self.bool_const(*val), "");
                self.match_constant(value, constant, on_fail);
            }
            Value::Atom(val) => {
                let constant = self.allocate_const(self.atom_const(val.to_owned()), "");
                self.match_constant(value, constant, on_fail);
            }
            Value::Character(val) => {
                let constant = self.allocate_const(self.char_const(*val), "");
                self.match_constant(value, constant, on_fail);
            }
            Value::Number(num) => {
                let constant = self.allocate_value("");
                self.number_const(constant, num);
                self.match_constant(value, constant, on_fail);
            }
            Value::Bits(bits) => {
                let constant = self.allocate_value("");
                self.bits_const(constant, bits);
                self.match_constant(value, constant, on_fail);
            }
            Value::String(string) => {
                let constant = self.allocate_value("");
                self.string_const(constant, string);
                self.match_constant(value, constant, on_fail);
            }
            Value::Application(app) => {
                self.compile_match_application(app, value, on_fail, bound_ids)?
            }
            Value::Wildcard => {}
            _ => todo!(),
        }

        if let Some(prev) = prev {
            self.overwrite_debug_location(prev);
        }

        Some(())
    }

    fn match_constant(
        &self,
        value: PointerValue<'ctx>,
        constant: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
    ) {
        let value_ref = self.use_temporary(value).unwrap();
        let is_match = self.trilogy_value_structural_eq(value_ref, constant, "");
        self.pm_cont_if(is_match, on_fail);
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
        let snapshot = self.snapshot_function_context();
        self.builder.position_at_end(fail);
        let on_fail = self.use_temporary(on_fail).unwrap();
        self.void_call_continuation(on_fail, "");
        self.builder.build_unreachable().unwrap();

        self.builder.position_at_end(cont);
        self.restore_function_context(snapshot);
        self.resume_continuation_point(&brancher);
        cont
    }

    fn compile_match_application(
        &self,
        application: &ir::Application,
        value: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
        bound_ids: &mut Vec<Id>,
    ) -> Option<()> {
        match &application.function.value {
            Value::Builtin(builtin) => {
                self.compile_match_apply_builtin(*builtin, &application.argument, value, on_fail)
            }
            Value::Application(app) => match &app.function.value {
                Value::Builtin(Builtin::Cons) => {
                    let value_ref = self.use_temporary(value).unwrap();
                    let tag = self.get_tag(value_ref, "");
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

                    let tuple = self.trilogy_tuple_assume(value_ref, "");
                    let left = self.allocate_value("");
                    self.bind_temporary(left);
                    self.trilogy_tuple_left(left, tuple);
                    self.match_pattern(&app.argument, left, on_fail, bound_ids)?;
                    if let Some(left) = self.use_owned_temporary(left) {
                        self.trilogy_value_destroy(left);
                    }

                    let value_ref = self.use_temporary(value).unwrap();
                    let tuple = self.trilogy_tuple_assume(value_ref, "");
                    let right = self.allocate_value("");
                    self.bind_temporary(right);
                    self.trilogy_tuple_right(right, tuple);
                    self.match_pattern(&application.argument, right, on_fail, bound_ids)?;
                    if let Some(right) = self.use_owned_temporary(right) {
                        self.trilogy_value_destroy(right);
                    }
                    Some(())
                }
                Value::Builtin(Builtin::Construct) => {
                    let value_ref = self.use_temporary(value).unwrap();
                    let tag = self.get_tag(value_ref, "");
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
                    self.bind_temporary(destructed);
                    self.destruct(destructed, value_ref);
                    let tuple = self.trilogy_tuple_assume(destructed, "");
                    let part = self.allocate_value("");
                    self.bind_temporary(part);
                    self.trilogy_tuple_left(part, tuple);
                    // We can be sure that the argument is just an atom constant, so won't invalidate
                    // the tuple reference
                    self.match_pattern(&application.argument, part, on_fail, bound_ids)?;
                    self.trilogy_value_destroy(part);
                    self.trilogy_tuple_right(part, tuple);
                    self.match_pattern(&app.argument, part, on_fail, bound_ids)?;
                    if let Some(temp) = self.use_owned_temporary(part) {
                        self.trilogy_value_destroy(temp);
                    }
                    if let Some(temp) = self.use_owned_temporary(destructed) {
                        self.trilogy_value_destroy(temp);
                    }
                    Some(())
                }
                _ => panic!("only builtins can be applied in pattern matching context"),
            },
            _ => panic!("only builtins can be applied in pattern matching context"),
        }
    }

    fn compile_match_apply_builtin(
        &self,
        builtin: Builtin,
        expression: &ir::Expression,
        value: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
    ) -> Option<()> {
        match builtin {
            Builtin::Typeof => {
                // TODO: we should restrict the expressions in this thing to be pins or constants... otherwise we do have to
                // handle branching...
                let expected_type = self.compile_expression(expression, "")?;
                let value = self.use_temporary(value).unwrap();
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
                self.match_constant(value, pinned, on_fail);
            }
            _ => todo!(),
        }
        Some(())
    }
}
