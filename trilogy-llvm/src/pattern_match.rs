use crate::{
    Codegen,
    types::{TAG_STRUCT, TAG_TUPLE},
};
use inkwell::{
    IntPredicate,
    basic_block::BasicBlock,
    values::{IntValue, PointerValue},
};
use num::{ToPrimitive, Zero};
use trilogy_ir::ir::{self, Builtin, Value};

impl<'ctx> Codegen<'ctx> {
    #[must_use = "must acknowledge continuation of control flow"]
    pub(crate) fn compile_pattern_match(
        &self,
        pattern: &ir::Expression,
        value: PointerValue<'ctx>,
    ) -> Option<()> {
        let prev = self.set_span(pattern.span);

        match &pattern.value {
            Value::Reference(id) => {
                let variable = self.variable(id);
                self.trilogy_value_clone_into(variable, value);
            }
            Value::Conjunction(conj) => {
                self.compile_pattern_match(&conj.0, value)?;
                self.compile_pattern_match(&conj.1, value)?;
            }
            Value::Disjunction(conj) => {
                // TODO: implement the branching here, right now it's just taking the first and ignoring the second
                // e.g. `let a:_ or _:a = 1:2`
                let function = self.get_function();
                let secondary = self.context.append_basic_block(function, "pm_disj_second");
                let on_success = self.context.append_basic_block(function, "pm_disj_cont");
                self.compile_pattern_match(&conj.0, value)?;
                self.builder.build_unconditional_branch(on_success).unwrap();

                self.builder.position_at_end(secondary);
                self.compile_pattern_match(&conj.1, value)?;
                self.builder.build_unconditional_branch(on_success).unwrap();

                self.builder.position_at_end(on_success);
            }
            Value::Unit => {
                let constant = self.allocate_const(self.unit_const(), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match);
            }
            Value::Boolean(val) => {
                let constant = self.allocate_const(self.bool_const(*val), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match);
            }
            Value::Atom(val) => {
                let constant = self.allocate_const(self.atom_const(val.to_owned()), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match);
            }
            Value::Character(val) => {
                let constant = self.allocate_const(self.char_const(*val), "");
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(is_match);
            }
            Value::Number(num) if num.value().im.is_zero() && num.value().re.is_integer() => {
                if let Some(int) = num.value().re.to_i64() {
                    let constant = self.allocate_const(self.int_const(int), "");
                    let is_match = self.trilogy_value_structural_eq(value, constant, "");
                    self.pm_cont_if(is_match);
                } else {
                    todo!("Support non-integers and large integers")
                }
            }
            Value::Bits(bits) => {
                let constant = self.allocate_value("");
                self.bits_const(constant, bits);
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.trilogy_value_destroy(constant);
                self.pm_cont_if(is_match);
            }
            Value::String(string) => {
                let constant = self.allocate_value("");
                self.string_const(constant, string);
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.trilogy_value_destroy(constant);
                self.pm_cont_if(is_match);
            }
            Value::Application(app) => self.compile_match_application(value, app)?,
            Value::Wildcard => {}
            _ => todo!(),
        }

        if let Some(prev) = prev {
            self.overwrite_debug_location(prev);
        }

        Some(())
    }

    fn pm_cont_if(&self, cond: IntValue<'ctx>) -> BasicBlock<'ctx> {
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
        let end = self.get_end("");
        self.void_call_continuation(end);

        self.builder.position_at_end(cont);
        self.resume_continuation_point(&brancher);
        cont
    }

    fn compile_match_application(
        &self,
        value: PointerValue<'ctx>,
        application: &ir::Application,
    ) -> Option<()> {
        match &application.function.value {
            Value::Builtin(builtin) => {
                self.compile_match_apply_builtin(value, *builtin, &application.argument)
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
                    self.pm_cont_if(is_tuple);
                    let tuple = self.trilogy_tuple_assume(value, "");
                    let part = self.allocate_value("");
                    self.trilogy_tuple_left(part, tuple);
                    self.compile_pattern_match(&app.argument, part)?;
                    self.trilogy_value_destroy(part);
                    self.trilogy_tuple_right(part, tuple);
                    self.compile_pattern_match(&application.argument, part)?;
                    self.trilogy_value_destroy(part);
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
                    self.pm_cont_if(is_struct);
                    let destructed = self.allocate_value("");
                    self.destruct(destructed, value);
                    let tuple = self.trilogy_tuple_assume(destructed, "");
                    let part = self.allocate_value("");
                    self.trilogy_tuple_right(part, tuple);
                    self.compile_pattern_match(&app.argument, part)?;
                    self.trilogy_value_destroy(part);
                    self.trilogy_tuple_left(part, tuple);
                    self.compile_pattern_match(&application.argument, part)?;
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
    ) -> Option<()> {
        match builtin {
            Builtin::Typeof => {
                let expected_type = self.compile_expression(expression, "")?;
                let tag = self.get_tag(value, "");
                let atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                let type_ptr = self.allocate_value("");
                self.trilogy_atom_init(type_ptr, atom);
                let cmp = self.trilogy_value_structural_eq(expected_type, type_ptr, "");
                // NOTE: atom does not require destruction, so type_ptr is ok
                self.pm_cont_if(cmp);
            }
            Builtin::Pin => {
                let pinned = self.compile_expression(expression, "pin")?;
                let cmp = self.trilogy_value_structural_eq(value, pinned, "");
                self.trilogy_value_destroy(pinned);
                self.pm_cont_if(cmp);
            }
            _ => todo!(),
        }
        Some(())
    }
}
