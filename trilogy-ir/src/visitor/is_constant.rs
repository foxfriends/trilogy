use super::{IrVisitable, IrVisitor};
use crate::ir;

pub struct IsConstant {
    result: bool,
}

impl IsConstant {}

impl IrVisitor for IsConstant {
    fn visit_identifier(&mut self, _value: &ir::Identifier) {
        self.result = false;
    }
}

pub trait MightBeConstant: IrVisitable + Sized {
    fn is_constant(&self) -> bool {
        let mut can_evaluate = IsConstant { result: true };
        self.visit(&mut can_evaluate);
        can_evaluate.result
    }
}

impl MightBeConstant for ir::Expression {}
