use super::{IrVisitable, IrVisitor};
use crate::ir;

pub struct IsConstant {
    result: bool,
    is_calling: bool,
}

impl IsConstant {}

impl IrVisitor for IsConstant {
    fn visit_identifier(&mut self, _value: &ir::Identifier) {
        if self.is_calling {
            self.result = false;
        }
    }

    fn visit_application(&mut self, node: &ir::Application) {
        self.is_calling = true;
        node.function.visit(self);
        self.is_calling = false;
        node.argument.visit(self);
    }
}

pub trait MightBeConstant: IrVisitable + Sized {
    fn is_constant(&self) -> bool {
        let mut can_evaluate = IsConstant {
            result: true,
            is_calling: false,
        };
        self.visit(&mut can_evaluate);
        can_evaluate.result
    }
}

impl MightBeConstant for ir::Expression {}
