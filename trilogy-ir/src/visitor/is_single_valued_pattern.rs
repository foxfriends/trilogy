use super::{IrVisitable, IrVisitor};
use crate::ir;

pub struct IsSingleValuedPattern {
    result: bool,
    is_pinned: bool,
}

impl IsSingleValuedPattern {}

impl IrVisitor for IsSingleValuedPattern {
    fn visit_wildcard(&mut self) {
        self.result = false;
    }

    fn visit_identifier(&mut self, _value: &ir::Identifier) {
        if !self.is_pinned {
            self.result = false;
        }
    }

    fn visit_builtin(&mut self, value: &ir::Builtin) {
        match value {
            ir::Builtin::Pin => self.is_pinned = true,
            ir::Builtin::Typeof => self.result = false,
            _ => {}
        }
    }

    fn visit_disjunction(&mut self, _node: &(ir::Expression, ir::Expression)) {
        self.result = false;
    }

    fn visit_application(&mut self, node: &ir::Application) {
        node.function.visit(self);
        node.argument.visit(self);
        self.is_pinned = false;
    }
}

pub trait MightBeSingleValued: IrVisitable + Sized {
    fn is_single_valued_pattern(&self) -> bool {
        let mut is_single_valued = IsSingleValuedPattern {
            result: true,
            is_pinned: false,
        };
        self.visit(&mut is_single_valued);
        is_single_valued.result
    }
}

impl MightBeSingleValued for ir::Expression {}
