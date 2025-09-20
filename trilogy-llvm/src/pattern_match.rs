use crate::codegen::{Codegen, Merger};
use crate::types::{TAG_ARRAY, TAG_NUMBER, TAG_STRUCT, TAG_TUPLE};
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
        self.compile_pattern_match_with_bindings(pattern, value, on_fail, &mut bound_ids)
    }

    #[must_use = "must acknowledge continuation of control flow"]
    pub(crate) fn compile_pattern_match_with_bindings(
        &self,
        pattern: &ir::Expression,
        value: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
        bound_ids: &mut Vec<Id>,
    ) -> Option<()> {
        self.bind_temporary(value);
        self.bind_temporary(on_fail);
        self.match_pattern(pattern, value, on_fail, bound_ids);
        Some(())
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
                if bound_ids.contains(&id.id) {
                    let pinned = self.variable(&id.id);
                    self.match_constant(value, pinned, on_fail);
                } else {
                    bound_ids.push(id.id.clone());
                    let variable = self.variable(&id.id);
                    let value_ref = self.use_temporary(value).unwrap();
                    self.trilogy_value_clone_into(variable, value_ref);
                }
            }
            Value::Conjunction(conj) => {
                self.match_pattern(&conj.0, value, on_fail, bound_ids)?;
                self.match_pattern(&conj.1, value, on_fail, bound_ids)?;
            }
            Value::Disjunction(disj) => {
                let on_success_function = self.add_continuation("pm.cont");
                let mut merger = Merger::default();

                let second_function = self.add_continuation("disj.snd");
                let (go_to_second, secondary_cp) =
                    self.capture_current_continuation(second_function, "disj.snd");
                let first_function = self.add_continuation("disj.fst");
                let (go_to_first, primary_cp) =
                    self.capture_current_continuation(first_function, "disj.fst");
                self.void_call_continuation(go_to_first);

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
                if self
                    .match_pattern(&disj.1, value, on_fail, bound_ids)
                    .is_some()
                {
                    let closure = self.void_continue_in_scope(on_success_function);
                    self.end_continuation_point_as_merge(&mut merger, closure);
                }

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
            Value::Wildcard => { /* always passes with no action */ }
            Value::Array(array) => self.match_array(array, value, on_fail, bound_ids)?,
            Value::Set(..) => {}
            Value::Record(..) => {}
            // Not patterns:
            Value::Pack(..) => unreachable!(),
            Value::Sequence(..) => unreachable!(),
            Value::Assignment(..) => unreachable!(),
            Value::Mapping(..) => unreachable!(),
            Value::Query(..) => unreachable!(),
            Value::While(..) => unreachable!(),
            Value::For(..) => unreachable!(),
            Value::Let(..) => unreachable!(),
            Value::IfElse(..) => unreachable!(),
            Value::Match(..) => unreachable!(),
            Value::Fn(..) => unreachable!(),
            Value::Do(..) => unreachable!(),
            Value::Qy(..) => unreachable!(),
            Value::Handled(..) => unreachable!(),
            Value::ModuleAccess(..) => unreachable!(),
            Value::Assert(..) => unreachable!(),
            Value::ArrayComprehension(..) => unreachable!(),
            Value::SetComprehension(..) => unreachable!(),
            Value::RecordComprehension(..) => unreachable!(),
            Value::End => unreachable!(),
            Value::Builtin(..) => unreachable!(),
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

        let brancher = self.branch_continuation_point();
        self.builder
            .build_conditional_branch(cond, cont, fail)
            .unwrap();
        self.builder.position_at_end(fail);
        let on_fail = self.use_temporary(on_fail).unwrap();
        self.void_call_continuation(on_fail);

        self.builder.position_at_end(cont);
        self.become_continuation_point(brancher);
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
            Value::Builtin(builtin) => self.compile_match_apply_builtin(
                *builtin,
                &application.argument,
                value,
                on_fail,
                bound_ids,
            ),
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
                Value::Builtin(Builtin::Glue) => todo!(),
                _ => panic!("only some operators are usable in pattern matching"),
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
        bound_ids: &mut Vec<Id>,
    ) -> Option<()> {
        match builtin {
            Builtin::Typeof => {
                let value = self.use_temporary(value).unwrap();
                let tag = self.get_tag(value, "typeof.tag");
                let atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                let type_ptr = self.allocate_value("typeof.atom");
                self.trilogy_atom_init(type_ptr, atom);
                self.bind_temporary(type_ptr);
                self.match_pattern(expression, type_ptr, on_fail, bound_ids)?;
            }
            Builtin::Pin => {
                // Because only identifiers can be pinned, we don't have to worry about handling branching mess here
                let pinned = self.compile_expression(expression, "pin")?;
                self.match_constant(value, pinned, on_fail);
            }
            Builtin::Negate => {
                let negated = self.allocate_value("negated");
                let value = self.use_temporary(value).unwrap();
                let is_number = self
                    .builder
                    .build_int_compare(
                        IntPredicate::EQ,
                        self.get_tag(value, ""),
                        self.tag_type().const_int(TAG_NUMBER, false),
                        "is_num",
                    )
                    .unwrap();
                self.pm_cont_if(is_number, on_fail);
                self.negate(negated, value);
                self.bind_temporary(negated);
                self.match_pattern(expression, negated, on_fail, bound_ids)?;
            }
            _ => panic!("only some operators are usable in pattern matching"),
        }
        Some(())
    }

    fn match_array(
        &self,
        array: &ir::Pack,
        value: PointerValue<'ctx>,
        on_fail: PointerValue<'ctx>,
        bound_ids: &mut Vec<Id>,
    ) -> Option<()> {
        let tag = self.get_tag(value, "");
        let is_array = self
            .builder
            .build_int_compare(
                IntPredicate::EQ,
                tag,
                self.tag_type().const_int(TAG_ARRAY, false),
                "is_array",
            )
            .unwrap();
        self.pm_cont_if(is_array, on_fail);

        if !array.values.iter().any(|el| el.is_spread) {
            let value = self.use_temporary(value).unwrap();
            let arr = self.trilogy_array_assume(value, "arr");
            let length = self.trilogy_array_len(arr, "len");
            let expected = self
                .context
                .i64_type()
                .const_int(array.values.len() as u64, false);
            let is_full = self
                .builder
                .build_int_compare(IntPredicate::EQ, length, expected, "")
                .unwrap();
            self.pm_cont_if(is_full, on_fail);
        }

        let mut spread = None;
        for (i, element) in array.values.iter().enumerate() {
            if element.is_spread {
                spread = Some((i, element));
                continue;
            }
            let value = self.use_temporary(value).unwrap();
            let array_value = self.trilogy_array_assume(value, "");
            if spread.is_some() {
                let len = self.trilogy_array_len(array_value, "");
                let distance = array.values.len() - i;
                let distance_rt = self.usize_type().const_int(distance as u64, false);
                let index = self.builder.build_int_sub(len, distance_rt, "").unwrap();
                let ith_value = self.allocate_value("");
                self.trilogy_array_at_dyn(ith_value, array_value, index);
                self.bind_temporary(ith_value);
                self.match_pattern(&element.expression, ith_value, on_fail, bound_ids)?;
                if let Some(temp) = self.use_owned_temporary(ith_value) {
                    self.trilogy_value_destroy(temp);
                }
            } else {
                let ith_value = self.allocate_value("");
                self.trilogy_array_at(ith_value, array_value, i);
                self.bind_temporary(ith_value);
                self.match_pattern(&element.expression, ith_value, on_fail, bound_ids)?;
                if let Some(temp) = self.use_owned_temporary(ith_value) {
                    self.trilogy_value_destroy(temp);
                }
            }
        }
        if let Some((i, spread)) = spread {
            let value = self.use_temporary(value).unwrap();
            let array_value = self.trilogy_array_assume(value, "");

            let len = self.trilogy_array_len(array_value, "");
            let distance = array.values.len() - (i + 1);
            let distance_rt = self.usize_type().const_int(distance as u64, false);
            let end = self.builder.build_int_sub(len, distance_rt, "").unwrap();

            let rest = self.allocate_value("");
            self.trilogy_array_slice(
                rest,
                array_value,
                self.usize_type().const_int(i as u64, false),
                end,
            );
            self.bind_temporary(rest);
            self.match_pattern(&spread.expression, rest, on_fail, bound_ids)?;
            if let Some(temp) = self.use_owned_temporary(rest) {
                self.trilogy_value_destroy(temp);
            }
        }
        Some(())
    }
}
