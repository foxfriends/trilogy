use crate::codegen::Codegen;
use inkwell::AddressSpace;
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
                let result = self.compile_expression(expression, name)?;
                let return_cont = self.get_return("");
                self.call_known_continuation(return_cont, result);
                None
            }
            Builtin::Exit => {
                let result = self.compile_expression(expression, name)?;
                _ = self.exit(result);
                None
            }
            Builtin::Typeof => {
                let argument = self.compile_expression(expression, "")?;
                let out = self.allocate_value(name);
                // The atom table is specifically defined so that a value's tag
                // lines up with it's typeof atom
                let tag = self.get_tag(argument, "");
                let raw_atom = self
                    .builder
                    .build_int_z_extend(tag, self.context.i64_type(), "")
                    .unwrap();
                self.trilogy_atom_init(out, raw_atom);
                self.trilogy_value_destroy(argument);
                Some(out)
            }
            Builtin::Not => {
                let argument = self.compile_expression(expression, "")?;
                let out = self.allocate_value(name);
                self.not(out, argument);
                self.trilogy_value_destroy(argument);
                Some(out)
            }
            Builtin::Yield => {
                let effect = self.compile_expression(expression, name)?;
                Some(self.call_yield(effect, name))
            }
            Builtin::Cancel => {
                let value = self.compile_expression(expression, name)?;
                let cancel = self.get_cancel("");
                self.call_known_continuation(cancel, value);
                None
            }
            Builtin::Resume => {
                let value = self.compile_expression(expression, name)?;
                Some(self.call_resume(value, name))
            }
            Builtin::Break => {
                let result = self.compile_expression(expression, name)?;
                let break_cont = self.get_break("");
                self.call_known_continuation(break_cont, result);
                None
            }
            Builtin::Continue => {
                let result = self.compile_expression(expression, name)?;
                self.call_continue(result, "");
                None
            }
            Builtin::ToString => {
                let value = self.compile_expression(expression, name)?;
                let string = self.to_string(value, "");
                Some(string)
            }
            Builtin::Negate => todo!(),
            Builtin::Invert => todo!(),
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
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.structural_eq(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::StructuralInequality => {
                let lhs = self.compile_expression(lhs, "sne.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "sne.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.structural_neq(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::ReferenceEquality => {
                let lhs = self.compile_expression(lhs, "req.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "req.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.referential_eq(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::ReferenceInequality => {
                let lhs = self.compile_expression(lhs, "rne.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "rne.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.referential_neq(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Access => {
                let lhs = self.compile_expression(lhs, "acc.c")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "acc.i")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.member_access(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Cons => {
                let lhs = self.compile_expression(lhs, "cons.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "cons.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.trilogy_tuple_init_new(out, lhs, rhs);
                Some(out)
            }
            Builtin::Construct => {
                let lhs = self.compile_expression(lhs, "struct.val")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "")?;
                let lhs = self.use_temporary(lhs).unwrap();
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
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.glue(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Lt => {
                let lhs = self.compile_expression(lhs, "lt.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "lt.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.lt(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Gt => {
                let lhs = self.compile_expression(lhs, "gt.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "gt.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.gt(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Leq => {
                let lhs = self.compile_expression(lhs, "lte.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "lte.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.lte(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Geq => {
                let lhs = self.compile_expression(lhs, "gte.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "gte.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.gte(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Add => {
                let lhs = self.compile_expression(lhs, "add.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "add.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.add(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Subtract => {
                let lhs = self.compile_expression(lhs, "sub.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "sub.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.sub(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Multiply => {
                let lhs = self.compile_expression(lhs, "mul.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "mul.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.mul(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Divide => {
                let lhs = self.compile_expression(lhs, "div.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "div.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.div(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::Or => self.compile_or(lhs, rhs, name),
            Builtin::And => self.compile_and(lhs, rhs, name),
            Builtin::Remainder => todo!(),
            Builtin::Power => todo!(),
            Builtin::IntDivide => todo!(),
            Builtin::BitwiseAnd => {
                let lhs = self.compile_expression(lhs, "b_and.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "b_and.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.bitwise_and(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::BitwiseOr => {
                let lhs = self.compile_expression(lhs, "b_or.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "b_or.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.bitwise_or(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::BitwiseXor => {
                let lhs = self.compile_expression(lhs, "b_xor.lhs")?;
                self.bind_temporary(lhs);
                let rhs = self.compile_expression(rhs, "b_xor.rhs")?;
                let lhs = self.use_temporary(lhs).unwrap();
                let out = self.allocate_value(name);
                self.bitwise_xor(out, lhs, rhs);
                self.trilogy_value_destroy(lhs);
                self.trilogy_value_destroy(rhs);
                Some(out)
            }
            Builtin::LeftShift => todo!(),
            Builtin::LeftShiftExtend => todo!(),
            Builtin::LeftShiftContract => todo!(),
            Builtin::RightShift => todo!(),
            Builtin::RightShiftExtend => todo!(),
            Builtin::RightShiftContract => todo!(),
            Builtin::Compose => todo!(),
            Builtin::RCompose => todo!(),
            Builtin::Pipe => todo!(),
            Builtin::RPipe => todo!(),
            Builtin::Sequence => todo!(),
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
                let return_to = self.get_return(name);
                self.trilogy_callable_promote(
                    return_to,
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                    self.get_yield(""),
                    self.get_cancel(""),
                    self.get_resume(""),
                    self.get_break(""),
                    self.get_continue(""),
                );
                return_to
            }
            Builtin::Cancel => self.get_cancel(name),
            Builtin::Resume => self.get_resume(name),
            Builtin::Break => {
                let break_to = self.get_break(name);
                self.trilogy_callable_promote(
                    break_to,
                    self.get_return(""),
                    self.get_yield(""),
                    self.get_cancel(""),
                    self.get_resume(""),
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                );
                break_to
            }
            Builtin::Continue => {
                let continue_to = self.get_continue(name);
                self.trilogy_callable_promote(
                    continue_to,
                    self.get_return(""),
                    self.get_yield(""),
                    self.get_cancel(""),
                    self.get_resume(""),
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                    self.context.ptr_type(AddressSpace::default()).const_null(),
                );
                continue_to
            }
            Builtin::Access => todo!(),
            Builtin::And => todo!(),
            Builtin::Or => todo!(),
            Builtin::Add => todo!(),
            Builtin::Subtract => todo!(),
            Builtin::Multiply => todo!(),
            Builtin::Divide => todo!(),
            Builtin::Remainder => todo!(),
            Builtin::Power => todo!(),
            Builtin::IntDivide => todo!(),
            Builtin::StructuralEquality => todo!(),
            Builtin::StructuralInequality => todo!(),
            Builtin::ReferenceEquality => todo!(),
            Builtin::ReferenceInequality => todo!(),
            Builtin::Lt => todo!(),
            Builtin::Gt => todo!(),
            Builtin::Leq => todo!(),
            Builtin::Geq => todo!(),
            Builtin::BitwiseAnd => todo!(),
            Builtin::BitwiseOr => todo!(),
            Builtin::BitwiseXor => todo!(),
            Builtin::LeftShift => todo!(),
            Builtin::LeftShiftExtend => todo!(),
            Builtin::LeftShiftContract => todo!(),
            Builtin::RightShift => todo!(),
            Builtin::RightShiftExtend => todo!(),
            Builtin::RightShiftContract => todo!(),
            Builtin::Cons => todo!(),
            Builtin::Glue => todo!(),
            Builtin::Compose => todo!(),
            Builtin::RCompose => todo!(),
            Builtin::Pipe => todo!(),
            Builtin::RPipe => todo!(),
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
