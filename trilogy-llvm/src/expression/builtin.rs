use crate::codegen::Codegen;
use inkwell::AddressSpace;
use inkwell::values::BasicValue;
use inkwell::values::PointerValue;
use trilogy_ir::ir::{self, Builtin};

impl<'ctx> Codegen<'ctx> {
    pub(super) fn compile_apply_unary(
        &self,
        builtin: Builtin,
        expression: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        match builtin {
            Builtin::Return => {
                let result = self.compile_expression(expression, "retval")?;
                let return_cont = self.get_return("return");
                self.call_known_continuation(return_cont, result);
                None
            }
            Builtin::Exit => {
                let result = self.compile_expression(expression, "exit_code")?;
                _ = self.exit(result);
                None
            }
            Builtin::Typeof => {
                let argument = self.compile_expression(expression, "typeof_arg")?;
                let out = self.allocate_value(name);
                // The atom table is specifically defined so that a value's tag
                // lines up with it's typeof atom
                let tag = self.get_tag(argument, "typeof_arg_tag");
                let raw_atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "raw_atom")
                    .unwrap();
                self.trilogy_atom_init(out, raw_atom);
                Some(out)
            }
            Builtin::Not => {
                let argument = self.compile_expression(expression, "not_arg")?;
                let out = self.allocate_value(name);
                self.boolean_not(out, argument);
                Some(out)
            }
            Builtin::Yield => {
                let effect = self.compile_expression(expression, "effect")?;
                Some(self.call_yield(effect, name))
            }
            Builtin::Cancel => {
                let value = self.compile_expression(expression, "cancel_arg")?;
                let cancel_clone = self.allocate_value("");
                self.trilogy_value_clone_into(cancel_clone, self.get_cancel());
                self.call_known_continuation(cancel_clone, value);
                None
            }
            Builtin::Resume => {
                let value = self.compile_expression(expression, "resume_arg")?;
                Some(self.call_resume(value, name))
            }
            Builtin::Break => {
                let result = self.compile_expression(expression, "break_arg")?;
                let break_cont = self.get_break();
                self.call_known_continuation(break_cont, result);
                None
            }
            Builtin::Continue => {
                let result = self.compile_expression(expression, "continue_arg")?;
                self.call_known_continuation(self.get_continue(), result);
                None
            }
            Builtin::ToString => {
                let value = self.compile_expression(expression, "to_string_arg")?;
                Some(self.to_string(value, name))
            }
            Builtin::Negate => {
                let value = self.compile_expression(expression, "negate_arg")?;
                let out = self.allocate_value(name);
                self.negate(out, value);
                Some(out)
            }
            Builtin::Invert => {
                let value = self.compile_expression(expression, "invert_arg")?;
                let out = self.allocate_value(name);
                self.invert(out, value);
                Some(out)
            }
            // Non-unary operators
            Builtin::Is => unreachable!(),
            Builtin::Pin => unreachable!(),
            Builtin::Remainder => unreachable!(),
            Builtin::Power => unreachable!(),
            Builtin::IntDivide => unreachable!(),
            Builtin::BitwiseAnd => unreachable!(),
            Builtin::BitwiseOr => unreachable!(),
            Builtin::BitwiseXor => unreachable!(),
            Builtin::LeftShift => unreachable!(),
            Builtin::LeftShiftExtend => unreachable!(),
            Builtin::LeftShiftContract => unreachable!(),
            Builtin::RightShift => unreachable!(),
            Builtin::RightShiftExtend => unreachable!(),
            Builtin::RightShiftContract => unreachable!(),
            Builtin::Compose => unreachable!(),
            Builtin::RCompose => unreachable!(),
            Builtin::Pipe => unreachable!(),
            Builtin::RPipe => unreachable!(),
            Builtin::Sequence => unreachable!(),
            Builtin::Access => unreachable!(),
            Builtin::Add => unreachable!(),
            Builtin::Subtract => unreachable!(),
            Builtin::Multiply => unreachable!(),
            Builtin::StructuralEquality => unreachable!(),
            Builtin::StructuralInequality => unreachable!(),
            Builtin::ReferenceEquality => unreachable!(),
            Builtin::ReferenceInequality => unreachable!(),
            Builtin::Lt => unreachable!(),
            Builtin::Gt => unreachable!(),
            Builtin::Leq => unreachable!(),
            Builtin::Geq => unreachable!(),
            Builtin::Cons => unreachable!(),
            Builtin::Glue => unreachable!(),
            Builtin::Construct => unreachable!(),
            Builtin::And => unreachable!(),
            Builtin::Or => unreachable!(),
            Builtin::Divide => unreachable!(),
        }
    }

    pub(super) fn compile_apply_binary(
        &self,
        builtin: Builtin,
        lhs: &ir::Expression,
        rhs: &ir::Expression,
        name: &str,
    ) -> Option<PointerValue<'ctx>> {
        match builtin {
            Builtin::StructuralEquality => {
                let lhs = self.compile_expression(lhs, "seq.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "seq.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.structural_eq(out, lhs, rhs);
                Some(out)
            }
            Builtin::StructuralInequality => {
                let lhs = self.compile_expression(lhs, "sne.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "sne.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.structural_neq(out, lhs, rhs);
                Some(out)
            }
            Builtin::ReferenceEquality => {
                let lhs = self.compile_expression(lhs, "req.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "req.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.referential_eq(out, lhs, rhs);
                Some(out)
            }
            Builtin::ReferenceInequality => {
                let lhs = self.compile_expression(lhs, "rne.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "rne.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.referential_neq(out, lhs, rhs);
                Some(out)
            }
            Builtin::Access => {
                let lhs = self.compile_expression(lhs, "acc.c")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "acc.i")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                Some(self.member_access(lhs, rhs, ""))
            }
            Builtin::Cons => {
                let lhs = self.compile_expression(lhs, "cons.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "cons.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.trilogy_tuple_init_take(out, lhs, rhs);
                Some(out)
            }
            Builtin::Construct => {
                let lhs = self.compile_expression(lhs, "struct.val")?;
                let rhs = self.compile_expression(rhs, "")?;
                let tag = self.trilogy_atom_untag(rhs, "struct.tag");
                self.trilogy_value_destroy(rhs);
                let out = self.allocate_value(name);
                self.trilogy_struct_init_new(out, tag, lhs);
                Some(out)
            }
            Builtin::Glue => {
                let lhs = self.compile_expression(lhs, "glue.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "glue.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.glue(out, lhs, rhs);
                Some(out)
            }
            Builtin::Lt => {
                let lhs = self.compile_expression(lhs, "lt.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "lt.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.lt(out, lhs, rhs);
                Some(out)
            }
            Builtin::Gt => {
                let lhs = self.compile_expression(lhs, "gt.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "gt.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.gt(out, lhs, rhs);
                Some(out)
            }
            Builtin::Leq => {
                let lhs = self.compile_expression(lhs, "lte.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "lte.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.lte(out, lhs, rhs);
                Some(out)
            }
            Builtin::Geq => {
                let lhs = self.compile_expression(lhs, "gte.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "gte.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.gte(out, lhs, rhs);
                Some(out)
            }
            Builtin::Add => {
                let lhs = self.compile_expression(lhs, "add.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "add.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.add(out, lhs, rhs);
                Some(out)
            }
            Builtin::Subtract => {
                let lhs = self.compile_expression(lhs, "sub.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "sub.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.subtract(out, lhs, rhs);
                Some(out)
            }
            Builtin::Multiply => {
                let lhs = self.compile_expression(lhs, "mul.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "mul.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.multiply(out, lhs, rhs);
                Some(out)
            }
            Builtin::Divide => {
                let lhs = self.compile_expression(lhs, "div.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "div.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.divide(out, lhs, rhs);
                Some(out)
            }
            Builtin::Remainder => {
                let lhs = self.compile_expression(lhs, "rem.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "rem.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.rem(out, lhs, rhs);
                Some(out)
            }
            Builtin::Power => {
                let lhs = self.compile_expression(lhs, "pow.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "pow.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.power(out, lhs, rhs);
                Some(out)
            }
            Builtin::IntDivide => {
                let lhs = self.compile_expression(lhs, "intdiv.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "intdiv.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.int_divide(out, lhs, rhs);
                Some(out)
            }
            Builtin::Or => self.compile_or(lhs, rhs, name),
            Builtin::And => self.compile_and(lhs, rhs, name),
            Builtin::BitwiseAnd => {
                let lhs = self.compile_expression(lhs, "b_and.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "b_and.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.bitwise_and(out, lhs, rhs);
                Some(out)
            }
            Builtin::BitwiseOr => {
                let lhs = self.compile_expression(lhs, "b_or.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "b_or.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.bitwise_or(out, lhs, rhs);
                Some(out)
            }
            Builtin::BitwiseXor => {
                let lhs = self.compile_expression(lhs, "b_xor.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "b_xor.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.bitwise_xor(out, lhs, rhs);
                Some(out)
            }
            Builtin::LeftShift => {
                let lhs = self.compile_expression(lhs, "lsh.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "lsh.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.shift_left(out, lhs, rhs);
                Some(out)
            }
            Builtin::LeftShiftExtend => {
                let lhs = self.compile_expression(lhs, "lshex.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "lshex.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.shift_left_extend(out, lhs, rhs);
                Some(out)
            }
            Builtin::LeftShiftContract => {
                let lhs = self.compile_expression(lhs, "lshcon.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "lshcon.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.shift_left_contract(out, lhs, rhs);
                Some(out)
            }
            Builtin::RightShift => {
                let lhs = self.compile_expression(lhs, "rsh.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "rsh.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.shift_right(out, lhs, rhs);
                Some(out)
            }
            Builtin::RightShiftExtend => {
                let lhs = self.compile_expression(lhs, "rshex.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "rshex.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.shift_right_extend(out, lhs, rhs);
                Some(out)
            }
            Builtin::RightShiftContract => {
                let lhs = self.compile_expression(lhs, "rshcon.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "rshcon.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                let out = self.allocate_value(name);
                self.shift_right_contract(out, lhs, rhs);
                Some(out)
            }
            Builtin::Compose => {
                let lhs = self.compile_expression(lhs, "compose.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "compose.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                Some(self.compose(lhs, rhs, name))
            }
            Builtin::RCompose => {
                let lhs = self.compile_expression(lhs, "rcompose.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "rcompose.rhs")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                Some(self.compose(rhs, lhs, name))
            }
            Builtin::Pipe => {
                let lhs = self.compile_expression(lhs, "pipe.arg")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "pipe.fn")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                Some(self.apply_function(rhs, lhs, name))
            }
            Builtin::RPipe => {
                let lhs = self.compile_expression(lhs, "pipe.fn")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "pipe.arg")?;
                let lhs = self.use_temporary_clone(lhs).unwrap();
                Some(self.apply_function(lhs, rhs, name))
            }
            Builtin::Sequence => {
                let lhs = self.compile_expression(lhs, "")?;
                self.trilogy_value_destroy(lhs);
                self.compile_expression(rhs, name)
            }
            // Non-binary operators
            Builtin::ToString => unreachable!(),
            Builtin::Negate => unreachable!(),
            Builtin::Not => unreachable!(),
            Builtin::Invert => unreachable!(),
            Builtin::Is => unreachable!(),
            Builtin::Typeof => unreachable!(),
            Builtin::Pin => unreachable!(),
            Builtin::Yield => unreachable!(),
            Builtin::Exit => unreachable!(),
            Builtin::Resume => unreachable!(),
            Builtin::Cancel => unreachable!(),
            Builtin::Return => unreachable!(),
            Builtin::Break => unreachable!(),
            Builtin::Continue => unreachable!(),
        }
    }

    pub(super) fn reference_builtin(&self, builtin: Builtin, name: &str) -> PointerValue<'ctx> {
        match builtin {
            Builtin::Return => {
                let return_to = self.get_return_temporary();
                self.trilogy_callable_promote(
                    return_to,
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                    self.get_next(""),
                    self.get_done(""),
                );

                let function = self.add_continuation("captured_return");
                let target = self.allocate_value(name);
                let closure = self
                    .builder
                    .build_alloca(self.value_type(), "TEMP_CLOSURE")
                    .unwrap();

                self.trilogy_callable_init_do(target, 1, closure, function);

                let here = self.builder.get_insert_block().unwrap();
                let snapshot = self.snapshot_function_context();

                let shadow = self.shadow_continuation_point();
                let capture =
                    self.capture_contination_point(closure.as_instruction_value().unwrap());

                self.become_continuation_point(capture);
                self.begin_next_function(function);
                let return_to = self.use_temporary_clone(return_to).unwrap();
                self.call_known_continuation(return_to, self.get_continuation(""));

                self.builder.position_at_end(here);
                self.restore_function_context(snapshot);
                self.become_continuation_point(shadow);
                target
            }
            Builtin::Cancel => {
                let function = self.add_continuation("captured_cancel");
                let target = self.allocate_value(name);
                let closure = self
                    .builder
                    .build_alloca(self.value_type(), "TEMP_CLOSURE")
                    .unwrap();

                self.trilogy_callable_init_do(target, 1, closure, function);

                let here = self.builder.get_insert_block().unwrap();
                let snapshot = self.snapshot_function_context();

                let shadow = self.shadow_continuation_point();
                let capture =
                    self.capture_contination_point(closure.as_instruction_value().unwrap());

                self.become_continuation_point(capture);
                self.begin_next_function(function);
                let cancel = self.allocate_value("");
                self.trilogy_value_clone_into(cancel, self.get_cancel());
                self.call_known_continuation(cancel, self.get_continuation(""));

                self.builder.position_at_end(here);
                self.restore_function_context(snapshot);
                self.become_continuation_point(shadow);
                target
            }
            Builtin::Resume => {
                let function = self.add_continuation("captured_resume");

                let target = self.allocate_value(name);
                let closure = self
                    .builder
                    .build_alloca(self.value_type(), "TEMP_CLOSURE")
                    .unwrap();

                self.trilogy_callable_init_do(target, 1, closure, function);

                let here = self.builder.get_insert_block().unwrap();
                let snapshot = self.snapshot_function_context();

                let shadow = self.shadow_continuation_point();
                let capture =
                    self.capture_contination_point(closure.as_instruction_value().unwrap());

                self.become_continuation_point(capture);
                self.begin_next_function(function);
                self.call_resume(self.get_continuation(""), "");
                self.call_known_continuation(self.get_return("return"), self.get_continuation(""));

                self.builder.position_at_end(here);
                self.restore_function_context(snapshot);
                self.become_continuation_point(shadow);
                target
            }
            Builtin::Break => {
                let function = self.add_continuation("captured_break");
                let target = self.allocate_value(name);
                let closure = self
                    .builder
                    .build_alloca(self.value_type(), "TEMP_CLOSURE")
                    .unwrap();

                self.trilogy_callable_init_do(target, 1, closure, function);

                let here = self.builder.get_insert_block().unwrap();
                let snapshot = self.snapshot_function_context();

                let shadow = self.shadow_continuation_point();
                let capture =
                    self.capture_contination_point(closure.as_instruction_value().unwrap());

                self.become_continuation_point(capture);
                self.begin_next_function(function);
                let break_cont = self.get_break();
                self.call_known_continuation(break_cont, self.get_continuation(""));

                self.builder.position_at_end(here);
                self.restore_function_context(snapshot);
                self.become_continuation_point(shadow);
                target
            }
            Builtin::Continue => {
                let function = self.add_continuation("captured_continue");

                let target = self.allocate_value(name);
                let closure = self
                    .builder
                    .build_alloca(self.value_type(), "TEMP_CLOSURE")
                    .unwrap();

                self.trilogy_callable_init_do(target, 1, closure, function);

                let here = self.builder.get_insert_block().unwrap();
                let snapshot = self.snapshot_function_context();

                let shadow = self.shadow_continuation_point();
                let capture =
                    self.capture_contination_point(closure.as_instruction_value().unwrap());

                self.become_continuation_point(capture);
                self.begin_next_function(function);
                self.call_known_continuation(self.get_continue(), self.get_continuation(""));

                self.builder.position_at_end(here);
                self.restore_function_context(snapshot);
                self.become_continuation_point(shadow);
                target
            }
            Builtin::Access => self.reference_core("member_access"),
            Builtin::And => self.reference_core("boolean_and"),
            Builtin::Or => self.reference_core("boolean_or"),
            Builtin::Add => self.reference_core("add"),
            Builtin::Subtract => self.reference_core("subtract"),
            Builtin::Multiply => self.reference_core("multiply"),
            Builtin::Divide => self.reference_core("divide"),
            Builtin::Remainder => self.reference_core("rem"),
            Builtin::Power => self.reference_core("power"),
            Builtin::IntDivide => self.reference_core("int_divide"),
            Builtin::StructuralEquality => self.reference_core("structural_eq"),
            Builtin::StructuralInequality => self.reference_core("structural_neq"),
            Builtin::ReferenceEquality => self.reference_core("referential_eq"),
            Builtin::ReferenceInequality => self.reference_core("referential_neq"),
            Builtin::Lt => self.reference_core("lt"),
            Builtin::Gt => self.reference_core("gt"),
            Builtin::Leq => self.reference_core("lte"),
            Builtin::Geq => self.reference_core("gte"),
            Builtin::BitwiseAnd => self.reference_core("bitwise_and"),
            Builtin::BitwiseOr => self.reference_core("bitwise_or"),
            Builtin::BitwiseXor => self.reference_core("bitwise_xor"),
            Builtin::LeftShift => self.reference_core("shift_left"),
            Builtin::LeftShiftExtend => self.reference_core("shift_left_extend"),
            Builtin::LeftShiftContract => self.reference_core("shift_left_contract"),
            Builtin::RightShift => self.reference_core("shift_right"),
            Builtin::RightShiftExtend => self.reference_core("shift_right_extend"),
            Builtin::RightShiftContract => self.reference_core("shift_right_contract"),
            Builtin::Cons => self.reference_core("cons"),
            Builtin::Glue => self.reference_core("glue"),
            Builtin::Compose => self.reference_core("compose"),
            Builtin::RCompose => self.reference_core("rcompose"),
            Builtin::Pipe => self.reference_core("pipe"),
            Builtin::RPipe => self.reference_core("rpipe"),
            // Not referenceable operators
            Builtin::Sequence => unreachable!(),
            Builtin::ToString => unreachable!(),
            Builtin::Negate => unreachable!(),
            Builtin::Not => unreachable!(),
            Builtin::Invert => unreachable!(),
            Builtin::Construct => unreachable!(),
            Builtin::Is => unreachable!(),
            Builtin::Typeof => unreachable!(),
            Builtin::Pin => unreachable!(),
            Builtin::Yield => unreachable!(),
            Builtin::Exit => unreachable!(),
        }
    }
}
