use crate::{scope::Scope, Codegen};
use inkwell::{
    basic_block::BasicBlock,
    values::{IntValue, PointerValue},
};
use num::{ToPrimitive, Zero};
use trilogy_ir::ir::{self, Builtin, Value};

impl<'ctx> Codegen<'ctx> {
    #[must_use]
    pub(crate) fn compile_pattern_match(
        &self,
        scope: &mut Scope<'ctx>,
        pattern: &ir::Expression,
        value: PointerValue<'ctx>,
        on_fail: BasicBlock<'ctx>,
    ) -> Option<()> {
        self.set_span(pattern.span);
        match &pattern.value {
            Value::Reference(id) => {
                let variable = self.variable(scope, id);
                self.trilogy_value_clone_into(variable, value);
                self.builder.build_store(variable, value).unwrap();
            }
            Value::Conjunction(conj) => {
                self.compile_pattern_match(scope, &conj.0, value, on_fail)?;
                self.compile_pattern_match(scope, &conj.1, value, on_fail)?;
            }
            Value::Disjunction(conj) => {
                // TODO: this is supposed to be able to cause a branch... instead of just being `else`
                // e.g. `let a:_ or _:a = 1:2`
                let secondary = self
                    .context
                    .append_basic_block(scope.function, "pm_disj_second");
                let on_success = self
                    .context
                    .append_basic_block(scope.function, "pm_disj_cont");
                self.compile_pattern_match(scope, &conj.0, value, secondary)?;
                self.builder.build_unconditional_branch(on_success).unwrap();

                self.builder.position_at_end(secondary);
                self.compile_pattern_match(scope, &conj.1, value, on_fail)?;
                self.builder.build_unconditional_branch(on_success).unwrap();

                self.builder.position_at_end(on_success);
            }
            Value::Unit => {
                let constant = self.allocate_const(self.unit_const());
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(scope, is_match, on_fail);
            }
            Value::Boolean(val) => {
                let constant = self.allocate_const(self.bool_const(*val));
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(scope, is_match, on_fail);
            }
            Value::Atom(val) => {
                let constant = self.allocate_const(self.atom_const(val.to_owned()));
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(scope, is_match, on_fail);
            }
            Value::Character(val) => {
                let constant = self.allocate_const(self.char_const(*val));
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(scope, is_match, on_fail);
            }
            Value::Number(num) if num.value().im.is_zero() && num.value().re.is_integer() => {
                if let Some(int) = num.value().re.to_i64() {
                    let constant = self.allocate_const(self.int_const(int));
                    let is_match = self.trilogy_value_structural_eq(value, constant, "");
                    self.pm_cont_if(scope, is_match, on_fail);
                } else {
                    todo!("Support non-integers and large integers")
                }
            }
            Value::String(string) => {
                let constant = self.allocate_const(self.string_const(string));
                let is_match = self.trilogy_value_structural_eq(value, constant, "");
                self.pm_cont_if(scope, is_match, on_fail);
            }
            Value::Application(app) => {
                self.compile_match_application(scope, value, app, on_fail)?
            }
            _ => todo!(),
        }
        Some(())
    }

    fn pm_cont_if(
        &self,
        scope: &Scope<'ctx>,
        cond: IntValue<'ctx>,
        on_fail: BasicBlock<'ctx>,
    ) -> BasicBlock<'ctx> {
        let cont = self.context.append_basic_block(scope.function, "pm_cont");
        self.builder
            .build_conditional_branch(cond, cont, on_fail)
            .unwrap();
        self.builder.position_at_end(cont);
        cont
    }

    fn compile_match_application(
        &self,
        scope: &mut Scope<'ctx>,
        value: PointerValue<'ctx>,
        application: &ir::Application,
        on_fail: BasicBlock<'ctx>,
    ) -> Option<()> {
        match &application.function.value {
            Value::Builtin(builtin) => self.compile_match_apply_builtin(
                scope,
                value,
                *builtin,
                &application.argument,
                on_fail,
            ),
            _ => panic!("only builtins can be applied in pattern matching context"),
        }
    }

    fn compile_match_apply_builtin(
        &self,
        scope: &mut Scope<'ctx>,
        value: PointerValue<'ctx>,
        builtin: Builtin,
        expression: &ir::Expression,
        on_fail: BasicBlock<'ctx>,
    ) -> Option<()> {
        match builtin {
            Builtin::Typeof => {
                let expected_type = self.compile_expression(scope, expression)?;

                let tag = self.get_tag(value);
                let atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                let atom = self.raw_atom_value(atom);

                let cmp = self.trilogy_value_structural_eq(expected_type, atom, "");
                self.pm_cont_if(scope, cmp, on_fail);
            }
            Builtin::Pin => {
                let expected_value = self.compile_expression(scope, expression)?;
                let cmp = self.trilogy_value_structural_eq(expected_value, value, "");
                self.pm_cont_if(scope, cmp, on_fail);
            }
            _ => todo!(),
        }
        Some(())
    }
}
